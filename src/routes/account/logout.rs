//! Logout request.

use axum::extract::Form;
use axum::response::{IntoResponse, Redirect, Response, Result};

use crate::routes::log_error;

use serde::Deserialize;

use http::header::{self, HeaderValue};

use cookie::{Cookie, SameSite};

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
pub async fn post(Form(mut form): Form<LogoutForm>) -> Result<Response> {
    let redirect_to = form.redirect_to.take().unwrap_or_else(|| "/~/Index".into());

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
