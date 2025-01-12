//! Web app routes.

pub mod account;
pub mod page;

use std::fmt::Display;

use axum::{
    response::{ErrorResponse, Redirect},
    routing::{get, post},
    Router,
};
use http::StatusCode;
use tracing::error;

use crate::ServerState;

/// Creates a router with all the routes.
pub fn all() -> Router<ServerState> {
    Router::new()
        .nest("/~account", account())
        .route("/~/{*path}", get(page::show))
        .route("/~edit/{*path}", get(page::edit).post(page::post))
        .route("/", get(redirect_stray))
        .route("/~", get(redirect_stray))
        .route("/~/", get(redirect_stray))
}

/// Creates a router for account routes.
///
/// Routes that allow the user to authenticate with password.
pub fn account() -> Router<ServerState> {
    Router::new()
        .route(
            "/login",
            get(account::login::show).post(account::login::post),
        )
        .route("/logout", post(account::logout::post))
}

/// Redirects any stray requests to the index page
///
/// A stray request is a request that requests:
/// * The index page @ `/`
/// * The base url @ `/~`
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn redirect_stray() -> Redirect {
    Redirect::to("/~/Index")
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
