//! API error types.

use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use anyhow::{anyhow, Error as AnyError};

/// The login does not exist
pub const NO_LOGIN_FOUND: u32 = 1001;
/// Credentials were bad or mismatched.
pub const BAD_CREDENTIALS: u32 = 1002;
/// Authorization token was missing.
pub const MISSING_AUTHORIZATION: u32 = 1003;
/// Authorization token passed was bad.
pub const BAD_AUTHORIZATION: u32 = 1004;
/// The page or model was not found.
pub const NOT_FOUND: u32 = 1005;
/// Edits to this page have been protected.
pub const EDITS_FORBIDDEN: u32 = 1006;

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
            NO_LOGIN_FOUND => "no login found",
            BAD_CREDENTIALS => "bad or mismatched credentials",
            MISSING_AUTHORIZATION => "unauthorized",
            BAD_AUTHORIZATION => "bad authorization, suggest: clear cache",
            NOT_FOUND => "not found",
            EDITS_FORBIDDEN => "this page is protected",
            _ => "unknown error",
        };

        Error::new(code, message)
    }

    /// The message of the error.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The error's code.
    pub fn code(&self) -> u32 {
        self.code
    }

    /// Checks if the error was because the page wasn't found.
    pub fn not_found(&self) -> bool {
        self.code == NOT_FOUND
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
