//! Account authentication.

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

/// Server-only claims.
#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    /// The username to identify as.
    pub username: String,
}

/// Attempts password authentication for a username-password pair.
#[server(endpoint = "account/signin")]
pub async fn try_password_auth(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::ServerState;
    use axum::http::{header, HeaderValue};
    use cookie::Cookie;
    use hmac::{Hmac, Mac};
    use jwt::SignWithKey;
    use leptos_axum::ResponseOptions;
    use sha2::Sha256;

    // get signing key
    let state = expect_context::<ServerState>();

    // TODO: do proper password auth
    let key: Hmac<Sha256> = Hmac::new_from_slice(state.jwt_secret.as_bytes())?;
    let claims = Claims {
        username: "frostu8".to_string(),
    };
    let token = claims.sign_with_key(&key)?;

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
    let try_password_auth = ServerAction::<TryPasswordAuth>::new();

    view! {
        <ActionForm action=try_password_auth>
            <label for="username">Username</label>
            <input type="text" id="username" name="username"/>
            <label for="password">Password</label>
            <input type="text" id="password" name="password"/>
            <input type="submit" value="Login"/>
        </ActionForm>
    }
}
