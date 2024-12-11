//! General server-only types and functions.

use std::sync::Arc;

use base16::encode_lower;

use rand::{rngs::StdRng, Rng, SeedableRng};

use jsonwebtoken::{DecodingKey, EncodingKey};

use sqlx::{pool::PoolOptions, PgPool};

use anyhow::Error;

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
    /// Will attempt to connect to the Postgres database at `db_connect_str`.
    pub async fn new(db_connect_str: &str) -> Result<ServerState, Error> {
        // establish database connection
        let pool = PoolOptions::new()
            // configure db
            .connect(db_connect_str)
            .await?;

        // randomly generate JWT secret
        let mut rng = StdRng::from_entropy();
        let mut bytes = [0u8; 256];
        rng.fill(&mut bytes);

        let keys = Arc::from(SigningKeys::new(&encode_lower(&bytes))?);

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
            encoding: EncodingKey::from_base64_secret(&secret)?,
            decoding: DecodingKey::from_base64_secret(&secret)?,
        })
    }
}
