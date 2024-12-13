//! General server-only types and functions.

use std::future::Future;
use std::sync::Arc;

use base16::encode_lower;

use rand::{rngs::StdRng, Rng, SeedableRng};

use jsonwebtoken::{DecodingKey, EncodingKey};

use sqlx::{pool::PoolOptions, PgPool};

use serde::Deserialize;

use anyhow::{bail, Error};

/// Server config.
///
/// Can construct a [`ServerState`] using [`ServerStateConfig::build`].
#[derive(Clone, Default, Deserialize, PartialEq)]
pub struct ServerStateConfig {
    database_url: Option<String>,
    signing_key: Option<String>,
}

impl ServerStateConfig {
    /// Creates a new `ServerStateConfig` with sensible defaults.
    pub fn new() -> ServerStateConfig {
        ServerStateConfig::default()
    }

    /// The database connection url.
    pub fn database_url(self, database_url: impl Into<String>) -> Self {
        Self {
            database_url: Some(database_url.into()),
            ..self
        }
    }

    /// The signing key to use when signing tokens. If no key is passed, will
    /// be randomly generated on startup.
    ///
    /// This will be an HMAC key.
    pub fn signing_key(self, key: impl Into<String>) -> Self {
        Self {
            signing_key: Some(key.into()),
            ..self
        }
    }

    /// Builds a [`ServerState`], establishing any needed connections and such.
    pub fn build(self) -> impl Future<Output = Result<ServerState, Error>> {
        ServerState::new(self)
    }
}

/// Shared server state.
///
/// Cheaply cloneable.
#[derive(Clone)]
pub struct ServerState {
    /// A database connection pool.
    pub pool: PgPool,
    /// The secret signing keys for tokens.
    ///
    /// This is randomly generated on app startup. This means that when the
    /// daemon restarts, old JWTs will be rejected.
    pub keys: Arc<SigningKeys>,
}

impl ServerState {
    /// Creates a new `ServerState`.
    ///
    /// See [`ServerStateConfig`] and [`ServerStateConfig::build`] on how to
    /// use this.
    pub async fn new(config: ServerStateConfig) -> Result<ServerState, Error> {
        // get url
        let Some(database_url) = config.database_url.as_ref() else {
            bail!("`DATABASE_URL` or database_url not set in config");
        };

        // establish database connection
        let pool = PoolOptions::new()
            // configure db
            .connect(database_url)
            .await?;

        // randomly generate JWT secret
        let keys = match config.signing_key.as_ref() {
            Some(key) => Arc::from(SigningKeys::new(key)?),
            None => {
                let key = random_signing_key();

                Arc::from(SigningKeys::new(&key)?)
            }
        };

        Ok(ServerState { pool, keys })
    }
}

/// Signing keys.
pub struct SigningKeys {
    /// The encoding key.
    pub encoding: EncodingKey,
    /// The decoding key.
    pub decoding: DecodingKey,
}

impl SigningKeys {
    /// Creates a new set of `SigningKeys` from a base64 secret.
    pub fn new(secret: &str) -> Result<SigningKeys, Error> {
        assert!(
            secret.len() == 512,
            "key is invalid length {}",
            secret.len()
        );

        Ok(SigningKeys {
            encoding: EncodingKey::from_base64_secret(secret)?,
            decoding: DecodingKey::from_base64_secret(secret)?,
        })
    }
}

/// Creates a random HMAC signing key and returns it as a [`String`]
pub fn random_signing_key() -> String {
    let mut rng = StdRng::from_entropy();
    let mut bytes = [0u8; 256];
    rng.fill(&mut bytes);

    encode_lower(&bytes)
}
