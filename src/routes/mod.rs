//! Web app routes.

pub mod account;
pub mod page;

use std::fmt::Display;

use axum::{
    response::ErrorResponse,
    routing::{get, post},
    Router,
};
use http::StatusCode;
use tracing::error;

use crate::ServerState;

/// Creates a router with all the routes.
pub fn all() -> Router<ServerState> {
    Router::new()
        .route(
            "/~account/login",
            get(account::login::show).post(account::login::post),
        )
        .route("/~account/logout", post(account::logout::post))
        .route("/~/{*path}", get(page::show).post(page::post))
        .route("/~edit/{*path}", get(page::edit))
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
