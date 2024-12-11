//! Account login.

#[cfg(feature = "ssr")]
use super::Claims;

use crate::error::Error as ApiError;

use leptos::prelude::*;

/// Attempts password authentication for a username-password pair.
#[server(endpoint = "account/signin")]
pub async fn password_auth(
    username: String,
    password: String,
) -> Result<(), ServerFnError<ApiError>> {
    use crate::ServerState;
    use axum::http::{header, HeaderValue};
    use chrono::Utc;
    use cookie::Cookie;
    use jsonwebtoken::{encode, Header};
    use leptos_axum::ResponseOptions;

    // get signing key
    let state = expect_context::<ServerState>();

    // TODO: do proper password auth

    let exp = (Utc::now().naive_utc() + chrono::naive::Days::new(1))
        .and_utc()
        .timestamp() as usize;

    let claims = Claims {
        sub: "frostu8".to_string(),
        exp,
    };

    let token = encode(&Header::default(), &claims, &state.keys.encoding)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    // set cookie
    let response = expect_context::<ResponseOptions>();
    let cookie = Cookie::build(("auth", token)).path("/");
    if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
        response.insert_header(header::SET_COOKIE, cookie);
    }

    Ok(())
}

/// Displays a login form.
#[component]
pub fn Login() -> impl IntoView {
    let password_auth = ServerAction::<PasswordAuth>::new();

    view! {
        <ActionForm action=password_auth>
            <label for="username">Username</label>
            <input type="text" id="username" name="username"/>
            <label for="password">Password</label>
            <input type="text" id="password" name="password"/>
            <input type="submit" value="Login"/>
        </ActionForm>
    }
}
