//! Account login.

#[cfg(feature = "ssr")]
use super::Claims;

use crate::error::Error as ApiError;

use leptos::prelude::*;
use leptos::Params;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;

/// Attempts password authentication for a username-password pair.
#[server(endpoint = "account/signin")]
pub async fn password_auth(
    username: String,
    password: String,
    #[server(default)] redirect_to: Option<String>,
) -> Result<(), ServerFnError<ApiError>> {
    use crate::{error, passwords::hash_password, schema::user::get_password_login, ServerState};
    use axum::http::{header, HeaderValue};
    use chrono::Utc;
    use cookie::{Cookie, SameSite};
    use leptos_axum::{redirect, ResponseOptions};

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

        let token = super::grant_token(&claims)?;

        // set cookie
        let response = expect_context::<ResponseOptions>();
        let cookie = Cookie::build(("auth", token))
            .path("/")
            .expires(Some(
                cookie::time::OffsetDateTime::from_unix_timestamp(exp as i64).unwrap(),
            ))
            .same_site(SameSite::Strict);

        #[cfg(not(debug_assertions))]
        let cookie = cookie.secure(true);

        if let Ok(cookie) = HeaderValue::from_str(&cookie.to_string()) {
            response.insert_header(header::SET_COOKIE, cookie);
        }

        if let Some(url) = redirect_to {
            redirect(&url);
        }

        Ok(())
    } else {
        Err(ApiError::from_code(error::BAD_CREDENTIALS).into())
    }
}

/// Displays a login form.
#[component]
pub fn LoginForm(#[prop(optional, into)] redirect_to: Signal<Option<String>>) -> impl IntoView {
    let password_auth = ServerAction::<PasswordAuth>::new();
    let login_result = password_auth.value();

    let err_msg = move || {
        let result = login_result.get();

        match result {
            Some(Err(err)) => err.to_string(),
            _ => unreachable!(),
        }
    };

    view! {
        <ActionForm action=password_auth attr:class="form-login">
            // TODO: error modals
            <Show when=move || { matches!(login_result.get(), Some(Err(_))) }>
                <p>{err_msg}</p>
            </Show>
            <label for="username">Username</label>
            <input type="text" id="username" name="username" />
            <label for="password">Password</label>
            <input type="password" id="password" name="password" />
            {move || {
                redirect_to
                    .with(|path| {
                        path.as_ref()
                            .map(|href| {
                                view! {
                                    <input type="hidden" name="redirect_to" value=href.clone() />
                                }
                            })
                    })
            }}
            <input type="submit" value="Login" />
        </ActionForm>
    }
}

/// [`Login`] page parameters.
#[derive(Debug, PartialEq, Params)]
pub struct LoginPageQuery {
    pub redirect_to: Option<String>,
}

/// The login page.
#[component]
pub fn Login() -> impl IntoView {
    let query = use_query::<LoginPageQuery>();

    let redirect_to = Signal::derive(move || {
        query
            .read()
            .as_ref()
            .ok()
            .and_then(|q| q.redirect_to.clone())
    });

    view! {
        <div class="login-form-container">
            <h1>"~/inferno"</h1>
            <LoginForm redirect_to />
        </div>
    }
}
