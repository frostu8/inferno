//! Error types.

use eyre::Report;

use axum::response::{IntoResponse, Response};

use http::StatusCode;

use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

/// The app error type.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
}

impl From<Report> for Error {
    fn from(value: Report) -> Self {
        Error {
            kind: ErrorKind::Other(value),
            message: None,
        }
    }
}

/// The inner error kind.
#[derive(Debug)]
pub enum ErrorKind {
    /// A catastrophic error occured.
    Other(Report),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(m) = self.message.as_ref() {
            f.write_str(m)?;
        }

        // Write inner message
        if let Some(err) = self.source() {
            if self.message.is_some() {
                f.write_str(": ")?;
            }
            write!(f, "{}", err)?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::Other(report) => report.source(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An internal server error occured.",
        )
            .into_response();

        if matches!(self.kind, ErrorKind::Other(_)) {
            response.extensions_mut().insert(Arc::new(self));
        }

        response
    }
}
