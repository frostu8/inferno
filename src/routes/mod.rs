//! Web app routes.

pub mod page;

use std::fmt::Display;

use axum::{response::ErrorResponse, routing::get, Router};
use http::StatusCode;
use tracing::error;

use crate::ServerState;

/// Creates a router with all the routes.
pub fn all() -> Router<ServerState> {
    Router::new().route("/~/{*path}", get(page::show))
}

/// Logs an internal server error to the console without showing it to the
/// user.
pub fn log_error<D>(err: D) -> ErrorResponse
where
    D: Display,
{
    error!("{}", err);
    ErrorResponse::from((
        StatusCode::INTERNAL_SERVER_ERROR,
        "An internal server error occured",
    ))
}
