//! Account authentication, management and creation.

pub mod claims;
pub mod session;

use axum::middleware::Next;
use claims::Claims;
use http::HeaderValue;

use crate::schema::user::{get_user, User};
use crate::ServerState;

use axum::body::Body;
use axum::extract::{FromRef, Request, State};

use http::{header, request::Parts, StatusCode};

use cookie::{Cookie, SameSite};

use tracing::error;

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr as _;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};

/// The cookie name for the access token.
pub const ACCESS_TOKEN_NAME: &str = "auth";
/// The cookie name for the refresh token.
pub const REFRESH_TOKEN_NAME: &str = "authr";

/// An extracted token.
#[derive(Clone, Debug)]
pub struct Token {
    /// The list of claims the user has.
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for Token
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(token) = parts.extensions.get::<Token>() {
            return Ok(token.clone());
        }

        let state = ServerState::from_ref(state);

        // fetch token
        let cookie = parts
            .headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|cookie| cookie.to_str().ok())
            .flat_map(|cookie| cookie.split(';'))
            .filter_map(|cookie| Cookie::from_str(cookie.trim()).ok())
            .find(|cookie| cookie.name() == ACCESS_TOKEN_NAME);

        if let Some(auth_cookie) = cookie {
            let token = Claims::decode(auth_cookie.value(), &state.keys)
                .map(|claims| Token { claims })
                .map_err(|_| Error::InvalidAuthorization)?;

            // add to extensions for caching
            parts.extensions.insert(token.clone());

            Ok(token)
        } else {
            Err(Error::MissingAuthorization)
        }
    }
}

/// Like [`Token`], but also fetches the user from the database.
#[derive(Clone, Debug)]
pub struct CurrentUser(User);

impl Deref for CurrentUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = Token::from_request_parts(parts, state).await?;
        let state = ServerState::from_ref(state);

        get_user(&state.pool, token.claims.sub())
            .await
            .map_err(Error::Db)
            .and_then(|user| user.ok_or_else(|| Error::UserNoLongerExists))
            .map(CurrentUser)
    }
}

/// Authentication middleware.
///
/// Handles pre-authentication for requests, along with exchanging for a new
/// access token if a refresh token is passed along with the request. This is
/// **apparently** bad practice, but inferno both acts as the authorization
/// server and the resource server, so at this point it's all up in the air. I
/// am not a security specialist.
pub async fn refresh_session_middleware(
    State(state): State<ServerState>,
    token: Result<Token, Error>,
    request: Request,
    next: Next,
) -> Response {
    let (mut parts, body) = request.into_parts();

    let new_claims = match token {
        Err(Error::MissingAuthorization) | Err(Error::InvalidAuthorization) => {
            // fetch token
            let cookie = parts
                .headers
                .get_all(header::COOKIE)
                .iter()
                .filter_map(|cookie| cookie.to_str().ok())
                .flat_map(|cookie| cookie.split(';'))
                .filter_map(|cookie| Cookie::from_str(cookie.trim()).ok())
                .find(|cookie| cookie.name() == REFRESH_TOKEN_NAME);

            if let Some(cookie) = cookie {
                // refresh token if refresh token was also passed
                match session::refresh(&state, cookie.value()).await {
                    Ok(claims) => Some(claims),
                    Err(session::Error::InvalidToken(_)) => None,
                    Err(err) => {
                        error!("error refreshing token: {}", err);
                        None
                    }
                }
            } else {
                None
            }
        }
        Err(err) => {
            error!("token database error: {}", err);
            None
        }
        Ok(..) => None,
    };

    if let Some(claims) = new_claims.as_ref() {
        parts.extensions.insert(Token {
            claims: claims.clone(),
        });
    }

    // rebuild request
    let request = Request::from_parts(parts, body);
    let mut response = next.run(request).await;

    // update access key token
    if let Some(claims) = new_claims {
        match claims.encode(&state.keys) {
            Ok(token) => {
                let cookie = Cookie::build((ACCESS_TOKEN_NAME, token))
                    .path("/")
                    .expires(Some(
                        cookie::time::OffsetDateTime::from_unix_timestamp(claims.exp() as i64)
                            .unwrap(),
                    ))
                    .same_site(SameSite::Strict);
                #[cfg(not(debug_assertions))]
                let cookie = cookie.secure(true);

                match HeaderValue::from_str(&cookie.to_string()) {
                    Ok(header) => {
                        response.headers_mut().append(header::SET_COOKIE, header);
                    }
                    Err(err) => {
                        error!("failed to encode token: {}", err);
                    }
                }
            }
            Err(err) => {
                error!("failed to encode token: {}", err);
            }
        }
    }

    response
}

/// An error that can occur during token extraction.
#[derive(Debug)]
pub enum Error {
    /// No auth cookie was passed, or the cookie was empty.
    MissingAuthorization,
    /// The content of the auth cookie was invalid.
    InvalidAuthorization,
    /// The token is valid, but points to a user that does not exist.
    UserNoLongerExists,
    /// Something wrong happened when accessing the database.
    Db(sqlx::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingAuthorization => f.write_str("authorization missing"),
            Error::InvalidAuthorization => f.write_str("authorization bad"),
            Error::UserNoLongerExists => f.write_str("token refers to a user that does not exist"),
            Error::Db(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Db(err) => Some(err),
            _ => None,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::MissingAuthorization
            | Error::InvalidAuthorization
            | Error::UserNoLongerExists => StatusCode::UNAUTHORIZED,
            Error::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}
