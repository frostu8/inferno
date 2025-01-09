//! Server shared state and config.

use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use base16::encode_lower;

use rand::{rngs::StdRng, Rng, SeedableRng};

use jsonwebtoken::{errors::Error as JwtError, DecodingKey, EncodingKey};

use sqlx::{pool::PoolOptions, PgPool};

use color_eyre::Section;
use eyre::{Report, WrapErr as _};

use serde::{Deserialize, Serialize};

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};

/// The default port the server is hosted on.
pub const DEFAULT_PORT: u16 = 4000;

/// Server config.
///
/// Can construct a [`ServerState`] using [`ServerConfig::build`].
#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct ServerConfig {
    port: u16,
    static_files_dir: PathBuf,
    #[serde(default)]
    database_url: Option<String>,
    #[serde(default)]
    signing_key: Option<String>,
}

impl ServerConfig {
    /// Creates a new `ServerConfig` with sensible defaults.
    pub fn new() -> ServerConfig {
        ServerConfig::default()
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
    pub fn build(self) -> impl Future<Output = Result<ServerState, Report>> {
        ServerState::new(self)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: DEFAULT_PORT,
            static_files_dir: PathBuf::from("./site"),
            database_url: None,
            signing_key: None,
        }
    }
}

/// Shared server state.
///
/// Cheaply cloneable.
#[derive(Clone)]
pub struct ServerState {
    /// The port the server is on.
    pub port: u16,
    /// Where static files should be served from.
    pub static_files_dir: PathBuf,
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
    /// See [`ServerConfig`] and [`ServerConfig::build`] on how to
    /// use this.
    pub async fn new(config: ServerConfig) -> Result<ServerState, Report> {
        let ServerConfig {
            port,
            static_files_dir,
            ..
        } = config;

        // get url
        let Some(database_url) = config.database_url.as_ref() else {
            return Err(Report::msg("`DATABASE_URL` not present")
                .suggestion("define `DATABASE_URL` with a valid postgres endpoint"));
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

                Arc::from(
                    SigningKeys::new(&key)
                        .wrap_err("failed to create signing keys")
                        .suggestion("signing key must be a valid HMAC secret")?,
                )
            }
        };

        Ok(ServerState {
            port,
            static_files_dir,
            pool,
            keys,
        })
    }
}

impl Debug for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerState")
            .field("port", &self.port)
            .field("static_files_dir", &self.static_files_dir)
            .finish_non_exhaustive()
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
    pub fn new(secret: &str) -> Result<SigningKeys, JwtError> {
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

/// Reads the config from the environment.
pub fn read_config() -> Result<ServerConfig, Report> {
    Figment::new()
        .merge(Serialized::defaults(ServerConfig::default()))
        .merge(Toml::file("inferno.toml"))
        .merge(Env::prefixed("INFERNO_"))
        .merge(Env::raw().only(&["DATABASE_URL", "PORT"]))
        .extract()
        .map_err(Report::from)
}
