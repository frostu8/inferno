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
    use crate::{error, passwords::hash_password, schema::user::get_password_login, ServerState};
    use axum::http::{header, HeaderValue};
    use chrono::Utc;
    use cookie::{Cookie, SameSite};
    use jsonwebtoken::{encode, Header};
    use leptos_axum::ResponseOptions;

    // get signing key
    let state = expect_context::<ServerState>();

    // fetch login by username
    let login = get_password_login(&state.pool, &username)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let login = login.ok_or_else(|| {
        ServerFnError::from(ApiError::new(
            error::NO_LOGIN_FOUND,
            format!("no login found for user `{}`", username),
        ))
    })?;

    // hash password
    let hashed_password = hash_password(&password, &login.salt);

    if hashed_password == login.password_hash {
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
        let cookie = Cookie::build(("auth", token))
            .path("/")
            .same_site(SameSite::Strict);

        #[cfg(not(debug_assertions))]
        let cookie = cookie.secure(true);

        if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
            response.insert_header(header::SET_COOKIE, cookie);
        }

        Ok(())
    } else {
        Err(ApiError::from_code(error::BAD_CREDENTIALS).into())
    }
}

/// Displays a login form.
#[component]
pub fn Login() -> impl IntoView {
    let password_auth = ServerAction::<PasswordAuth>::new();

    let login_result = password_auth.value();
    let err_msg = move || {
        login_result.with(|i| match i {
            Some(Err(err)) => err.to_string(),
            _ => unreachable!(),
        })
    };

    view! {
        <ActionForm action=password_auth class:form-login>
            // TODO: error modals
            <Show
                when=move || { matches!(login_result.get(), Some(Err(_))) }
            >
                <p>{err_msg}</p>
            </Show>
            <label for="username">Username</label>
            <input type="text" id="username" name="username"/>
            <label for="password">Password</label>
            <input type="text" id="password" name="password"/>
            <input type="submit" value="Login"/>
        </ActionForm>
    }
}
