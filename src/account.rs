//! Account authentication, management and creation.

use crate::schema::user::{get_user, User};
use crate::{ServerState, SigningKeys};

use axum::body::Body;
use axum::extract::FromRef;

use http::{header, request::Parts, StatusCode};

use cookie::Cookie;

use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr as _;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, Response};

use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, Algorithm, Header, TokenData, Validation,
};

/// Account claims.
///
/// This is normally encoded into a JSON Web Token and decoded on request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    /// The username to identify as.
    pub sub: String,
    /// The expiry timestamp.
    pub exp: usize,
}

/// Grants a token.
pub fn grant_token(keys: &SigningKeys, claims: &Claims) -> Result<String, JwtError> {
    let header = Header::new(Algorithm::HS256);

    encode(&header, claims, &keys.encoding)
}

/// Decodes a token passed as a cookie.
pub fn decode_token(keys: &SigningKeys, token: &str) -> Result<Claims, JwtError> {
    let validation = Validation::new(Algorithm::HS256);

    decode(token, &keys.decoding, &validation).map(|token: TokenData<Claims>| token.claims)
}

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
        let state = ServerState::from_ref(state);

        // fetch token
        let cookie = parts
            .headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|cookie| cookie.to_str().ok())
            .filter_map(|cookie| Cookie::from_str(cookie).ok())
            .find(|cookie| cookie.name() == "auth");

        if let Some(auth_cookie) = cookie {
            decode_token(&state.keys, auth_cookie.value())
                .map(|claims| Token { claims })
                .map_err(|_| Error::InvalidAuthorization)
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

        get_user(&state.pool, &token.claims.sub)
            .await
            .map_err(Error::Db)
            .and_then(|user| user.ok_or_else(|| Error::UserNoLongerExists))
            .map(CurrentUser)
    }
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

impl std::error::Error for Error {}

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
