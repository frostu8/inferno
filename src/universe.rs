//! Request context extractors.

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

use axum::body::Body;
use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use axum::response::{IntoResponse, Response};

use http::{header, request::Parts, StatusCode};

use crate::schema::universe::{get_universe_by_host, Universe};
use crate::ServerState;

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
            get_universe_by_host(&state.pool, host)
                .await
                .map_err(Error::Db)
                .and_then(|u| u.ok_or_else(|| Error::InvalidHost(host.into())))
                .map(CurrentUniverse)
        } else {
            Err(Error::NoHost)
        }
    }
}

impl<S> OptionalFromRequestParts<S> for CurrentUniverse
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let result =
            <CurrentUniverse as FromRequestParts<S>>::from_request_parts(parts, state).await;

        match result {
            Ok(universe) => Ok(Some(universe)),
            Err(Error::NoHost) | Err(Error::InvalidHost(..)) => Ok(None),
            Err(other) => Err(other),
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
    NoHost,
    /// Invalid host header.
    InvalidHost(String),
    /// Something wrong happened when accessing the database.
    Db(sqlx::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoHost => f.write_str("no Host header"),
            Error::InvalidHost(host) => write!(f, "Host {} not a universe", host),
            Error::Db(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::NoHost | Error::InvalidHost(..) => StatusCode::BAD_REQUEST,
            Error::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
