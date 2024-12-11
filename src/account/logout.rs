//! Account logout.
//!
//! This is just an endpoint that screws up the `auth` cookie so users have the
//! option to reauthenticate.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

/// Screws up the `auth` cookie and redirects the user.
#[server(endpoint = "account/logout", input = GetUrl)]
pub async fn logout_user() -> Result<(), ServerFnError> {
    use axum::http::{header, HeaderValue, StatusCode};
    use cookie::{Cookie, SameSite};
    use leptos_axum::ResponseOptions;

    let response = expect_context::<ResponseOptions>();

    let cookie = Cookie::build(("auth", ""))
        .path("/")
        .same_site(SameSite::Strict);

    if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }

    response.set_status(StatusCode::SEE_OTHER);
    response.insert_header(header::LOCATION, HeaderValue::from_static("/"));

    Ok(())
}
