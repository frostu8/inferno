//! Account claims for token access.

use serde::{Deserialize, Serialize};

use chrono::{TimeDelta, Utc};

use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, Algorithm, Header, TokenData, Validation,
};

use crate::SigningKeys;

/// This is normally encoded into a JSON Web Token and decoded on request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

impl Claims {
    /// Creates a [`ClaimsBuilder`] for a subject.
    pub fn for_sub(sub: impl Into<String>) -> ClaimsBuilder {
        ClaimsBuilder::new(sub)
    }

    /// The subject of the claim.
    pub fn sub(&self) -> &str {
        &self.sub
    }

    /// The expiry time of the claim.
    pub fn exp(&self) -> usize {
        self.exp
    }

    /// Grants a token.
    pub fn encode(&self, keys: &SigningKeys) -> Result<String, JwtError> {
        let header = Header::new(Algorithm::HS256);

        encode(&header, self, &keys.encoding)
    }

    /// Decodes a token passed as a cookie.
    pub fn decode(token: &str, keys: &SigningKeys) -> Result<Claims, JwtError> {
        let validation = Validation::new(Algorithm::HS256);

        decode(token, &keys.decoding, &validation).map(|token: TokenData<Claims>| token.claims)
    }
}

/// A builder for account claims.
#[derive(Debug)]
pub struct ClaimsBuilder {
    sub: String,
    exp: TimeDelta,
}

impl ClaimsBuilder {
    /// Creates a new `ClaimsBuilder` for a username.
    pub fn new(sub: impl Into<String>) -> ClaimsBuilder {
        ClaimsBuilder {
            sub: sub.into(),
            exp: TimeDelta::days(1),
        }
    }

    /// Sets the expiry time of the claims. By default, this is one day.
    pub fn exp(self, delta: TimeDelta) -> ClaimsBuilder {
        ClaimsBuilder { exp: delta, ..self }
    }

    /// Builds the claims for the [`Claims`] struct.
    pub fn build(self) -> Claims {
        let ClaimsBuilder { sub, exp } = self;

        Claims {
            sub,
            exp: (Utc::now().naive_utc() + exp).and_utc().timestamp() as usize,
        }
    }
}
