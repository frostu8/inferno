//! Logout request.

use axum::extract::{Form, State};
use axum::response::{IntoResponse, Redirect, Response, Result};

use crate::routes::log_error;
use crate::schema::session::dispose_session;
use crate::ServerState;

use serde::Deserialize;

use http::header::{self, HeaderMap, HeaderValue};

use cookie::{Cookie, SameSite};

use std::str::FromStr;

use tracing::instrument;

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct LogoutForm {
    #[serde(default)]
    redirect_to: Option<String>,
}

/// Handles a login request.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn post(
    headers: HeaderMap,
    State(state): State<ServerState>,
    Form(mut form): Form<LogoutForm>,
) -> Result<Response> {
    let redirect_to = form.redirect_to.take().unwrap_or_else(|| "/~/Index".into());

    // fetch session cookie to delete
    let cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|cookie| cookie.to_str().ok())
        .flat_map(|cookie| cookie.split(';'))
        .filter_map(|cookie| Cookie::from_str(cookie.trim()).ok())
        .find(|cookie| cookie.name() == crate::account::REFRESH_TOKEN_NAME);

    if let Some(cookie) = cookie {
        let hashed = crate::crypto::hash(cookie.value());

        dispose_session(&state.pool, &hashed)
            .await
            .map_err(log_error)?;
    }

    // set cookie
    let cookie = Cookie::build(("auth", ""))
        .path("/")
        .expires(Some(cookie::time::OffsetDateTime::now_utc()))
        .same_site(SameSite::Strict);

    #[cfg(not(debug_assertions))]
    let cookie = cookie.secure(true);

    let cookie = HeaderValue::from_str(&cookie.to_string()).map_err(log_error)?;

    let mut response = Redirect::to(&redirect_to).into_response();
    response.headers_mut().insert(header::SET_COOKIE, cookie);
    Ok(response)
}
