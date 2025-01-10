//! Request context extractors.

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use axum::body::Body;
use axum::extract::{FromRef, FromRequestParts};
use axum::response::{IntoResponse, Response};

use http::{header, request::Parts, StatusCode};

use crate::schema::universe::{get_global_universe, get_universe_by_host};
use crate::slug::Slug;
use crate::ServerState;

/// A in-database universe.
#[derive(Debug, sqlx::FromRow)]
pub struct Universe {
    /// The id of the universe.
    pub id: i32,
    /// The hostname to match when a request is made.
    ///
    /// This can be null for the default universe.
    pub host: Option<String>,
}

impl Universe {
    /// Creates a [`Locator`] for a [`Slug`] in the universe.
    pub fn locate<'a>(&self, path: &'a Slug) -> Locator<'a> {
        Locator {
            universe_id: self.id,
            path,
        }
    }
}

/// A locator for pages in a universe.
#[derive(Clone, Copy, Debug)]
pub struct Locator<'a> {
    /// The universe id.
    pub universe_id: i32,
    /// The page slug.
    pub path: &'a Slug,
}

/// An extractor for the universe.
#[derive(Debug)]
pub struct CurrentUniverse(Universe);

impl<S> FromRequestParts<S> for CurrentUniverse
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = ServerState::from_ref(state);

        // check host header
        if let Some(host) = parts
            .headers
            .get(header::HOST)
            .and_then(|s| s.to_str().ok())
        {
            let host = host.find(':').map(|idx| &host[..idx]).unwrap_or(host);

            // search for host in database
            let universe = get_universe_by_host(&state.pool, host)
                .await
                .map_err(Error::Db)?;

            if let Some(universe) = universe {
                Ok(CurrentUniverse(universe))
            } else {
                get_global_universe(&state.pool)
                    .await
                    .map(CurrentUniverse)
                    .map_err(Error::Db)
            }
        } else {
            get_global_universe(&state.pool)
                .await
                .map(CurrentUniverse)
                .map_err(Error::Db)
        }
    }
}

impl Deref for CurrentUniverse {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An error that can occur during extraction of [`CurrentUniverse`].
#[derive(Debug)]
pub enum Error {
    /// No host header passed.
    //NoHost,
    /// Invalid host header.
    //InvalidHost(String),
    /// Something wrong happened when accessing the database.
    Db(sqlx::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            //Error::NoHost => f.write_str("no Host header"),
            //Error::InvalidHost(host) => write!(f, "Host {} not a universe", host),
            Error::Db(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            //Error::NoHost | Error::InvalidHost(..) => StatusCode::BAD_REQUEST,
            Error::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
