//! Account authentication, management and creation.

pub mod login;

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

/// Account claims.
///
/// This is normally encoded into a JSON Web Token and decoded on request.
#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    /// The username to identify as.
    pub sub: String,
    /// The expiry timestamp.
    pub exp: usize,
}
