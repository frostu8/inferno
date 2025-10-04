//! Front-facing error types.

use eyre::Report;

use axum::response::{IntoResponse, Response};

use http::StatusCode;

use tracing::error;

/// The server error type.
///
/// The use of this type implies that there is some otherworldly phenomenon
/// causing a request error. Do not use this liberally.
#[derive(Debug)]
pub struct ServerError {
    /// The inner report type.
    pub inner: Report,
}

impl From<Report> for ServerError {
    fn from(inner: Report) -> ServerError {
        ServerError { inner }
    }
}

/// This creates an error report when this implementation is called.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        error!("{:?}", self.inner);

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An internal server error occured.",
        )
            .into_response()
    }
}
