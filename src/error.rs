//! API error types.

use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::{anyhow, Error as AnyError};

/// Credentials were bad.
pub const BAD_CREDENTIALS: u32 = 1001;

/// The main API error type.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    message: String,
    code: u32,
}

impl Error {
    /// Creates a new `Error`.
    pub fn new(code: u32, message: impl Into<String>) -> Error {
        Error {
            message: message.into(),
            code,
        }
    }

    /// Creates an error with a generic error message from a code.
    pub fn from_code(code: u32) -> Error {
        let message = match code {
            1001 => "bad or mismatched credentials",
            _ => "unknown error",
        };

        Error::new(code, message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}){}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl FromStr for Error {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("(") {
            let s = &s[1..];

            match s.find(")") {
                Some(idx) => {
                    let code = s[..idx].parse::<u32>()?;
                    let message = &s[idx + 1..];

                    Ok(Error::new(code, message))
                }
                None => Err(anyhow!("input does not have closing `)`")),
            }
        } else {
            Err(anyhow!("input does not begin with `(`"))
        }
    }
}
