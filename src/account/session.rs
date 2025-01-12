//! Session management.

use super::claims::Claims;

use crate::crypto::{generate_salt, hash};
use crate::schema::session::{create_session, get_session};
use crate::ServerState;

use std::fmt::{self, Display, Formatter};

/// Establishes a session for some claims.
///
/// Returns a set of claims and a session key.
pub async fn establish(state: &ServerState, username: &str) -> Result<(Claims, String), Error> {
    // generate a session key
    let key = generate_salt(64);
    let hashed_key = hash(&key);

    create_session(&state.pool, username, &hashed_key)
        .await
        .map_err(Error::Db)?;

    let claims = Claims::for_sub(username).build();

    Ok((claims, key))
}

/// Attempts to refresh a session from request parts.
///
/// Returns the new claims if successful.
pub async fn refresh(state: &ServerState, refresh_key: &str) -> Result<Claims, Error> {
    let hashed_key = hash(refresh_key);

    let session = get_session(&state.pool, &hashed_key)
        .await
        .map_err(Error::Db)?;

    if let Some(session) = session {
        // generate new claims
        Ok(Claims::for_sub(&session.username).build())
    } else {
        Err(Error::InvalidToken(refresh_key.to_owned()))
    }
}

/// An error that can occur during session management.
#[derive(Debug)]
pub enum Error {
    /// An invalid refresh token was given.
    InvalidToken(String),
    /// A database error occured.
    Db(sqlx::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidToken(token) => write!(f, "token \"{}\" invalid", token),
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
