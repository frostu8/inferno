//! Account logout.
//!
//! This is just an endpoint that screws up the `auth` cookie so users have the
//! option to reauthenticate.

use leptos::prelude::*;

/// Screws up the `auth` cookie and redirects the user.
#[server(endpoint = "account/logout")]
pub async fn logout_user(redirect_to: Option<String>) -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue};
    use cookie::{Cookie, SameSite};
    use leptos_axum::{redirect, ResponseOptions};

    let response = expect_context::<ResponseOptions>();

    let cookie = Cookie::build(("auth", ""))
        .path("/")
        .same_site(SameSite::Strict);

    if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }

    if let Some(url) = redirect_to {
        redirect(&url);
    }

    Ok(())
}
