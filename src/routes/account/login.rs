//! Login page and post request.

use axum::extract::{Form, Query, State};
use axum::response::{IntoResponse, Redirect, Response, Result};

use crate::account::{session, CurrentUser, Error as AccountError};
use crate::crypto::hash_password;
use crate::html::HtmlTemplate;
use crate::routes::log_error;
use crate::schema::user::get_password_login;
use crate::ServerState;

use serde::Deserialize;

use http::header::{self, HeaderValue};

use cookie::{Cookie, SameSite};

use tracing::instrument;

use askama::Template;

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct ShowLoginParams {
    #[serde(default)]
    redirect_to: Option<String>,
}

#[derive(Template)]
#[template(path = "account/login.html")]
struct ShowLoginTemplate {
    redirect_to: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct LoginForm {
    username: String,
    password: String,
    #[serde(default)]
    redirect_to: Option<String>,
}

/// Shows the login page.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn show(
    Query(mut opts): Query<ShowLoginParams>,
    // HACK this fixes some weird compile errors
    State(_): State<ServerState>,
    user: Result<CurrentUser, AccountError>,
) -> Response {
    let redirect_to = opts.redirect_to.take().unwrap_or_else(|| "/~/Index".into());

    if user.is_ok() {
        Redirect::to(&redirect_to).into_response()
    } else {
        HtmlTemplate::new(ShowLoginTemplate {
            redirect_to,
            error: None,
        })
        .into_response()
    }
}

/// Handles a login request.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn post(
    State(state): State<ServerState>,
    Form(mut form): Form<LoginForm>,
) -> Result<Response> {
    let redirect_to = form.redirect_to.take().unwrap_or_else(|| "/~/Index".into());

    let login = get_password_login(&state.pool, &form.username)
        .await
        .map_err(log_error)?;

    if let Some(login) = login {
        // try to check password
        let hashed_password = hash_password(&form.password, &login.salt);

        if hashed_password == login.password_hash {
            // establish session
            let (claims, refresh_key) = session::establish(&state, &form.username)
                .await
                .map_err(log_error)?;
            let token = claims.encode(&state.keys).map_err(log_error)?;

            // creat response
            let mut response = Redirect::to(&redirect_to).into_response();

            // set ACCESS_TOKEN_NAME cookie
            let cookie = Cookie::build((crate::account::ACCESS_TOKEN_NAME, token))
                .path("/")
                .expires(Some(
                    cookie::time::OffsetDateTime::from_unix_timestamp(claims.exp() as i64).unwrap(),
                ))
                .same_site(SameSite::Strict);
            #[cfg(not(debug_assertions))]
            let cookie = cookie.secure(true);
            let cookie = HeaderValue::from_str(&cookie.to_string()).map_err(log_error)?;
            response.headers_mut().append(header::SET_COOKIE, cookie);

            // set REFRESH_TOKEN_NAME cookie
            let cookie = Cookie::build((crate::account::REFRESH_TOKEN_NAME, refresh_key))
                .path("/")
                .expires(Some(
                    cookie::time::OffsetDateTime::now_utc() + cookie::time::Duration::days(90),
                ))
                .same_site(SameSite::Strict)
                .http_only(true);
            #[cfg(not(debug_assertions))]
            let cookie = cookie.secure(true);
            let cookie = HeaderValue::from_str(&cookie.to_string()).map_err(log_error)?;
            response.headers_mut().append(header::SET_COOKIE, cookie);

            Ok(response)
        } else {
            // no login with the username found
            Ok(HtmlTemplate::new(ShowLoginTemplate {
                redirect_to,
                error: Some("incorrect password".into()),
            })
            .into_response())
        }
    } else {
        // no login with the username found
        Ok(HtmlTemplate::new(ShowLoginTemplate {
            redirect_to,
            error: Some(format!("no login found for user \"{}\"", form.username)),
        })
        .into_response())
    }
}
