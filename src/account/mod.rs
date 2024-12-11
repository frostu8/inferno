//! Account authentication, management and creation.

pub mod login;

use login::Login;

use crate::error::Error as ApiError;
#[cfg(feature = "ssr")]
use crate::{error, ServerState};

use serde::{Deserialize, Serialize};

use leptos::prelude::*;

/// Account claims.
///
/// This is normally encoded into a JSON Web Token and decoded on request.
#[cfg(feature = "ssr")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Claims {
    /// The username to identify as.
    pub sub: String,
    /// The expiry timestamp.
    pub exp: usize,
}

/// Grants a token.
#[cfg(feature = "ssr")]
pub fn grant_token(claims: &Claims) -> Result<String, ServerFnError<ApiError>> {
    use crate::ServerState;
    use jsonwebtoken::{encode, Algorithm, Header};

    let state = expect_context::<ServerState>();

    let header = Header::new(Algorithm::HS256);

    encode(&header, claims, &state.keys.encoding)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Decodes a token passed as a cookie.
#[cfg(feature = "ssr")]
pub fn decode_token(token: &str) -> Result<Claims, ServerFnError<ApiError>> {
    use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, TokenData, Validation};

    let state = expect_context::<ServerState>();

    let validation = Validation::new(Algorithm::HS256);

    decode(token, &state.keys.decoding, &validation)
        .map(|token: TokenData<Claims>| token.claims)
        .map_err(|e| match e.kind() {
            ErrorKind::InvalidToken
            | ErrorKind::InvalidSignature
            | ErrorKind::MissingRequiredClaim(_)
            | ErrorKind::ExpiredSignature
            | ErrorKind::InvalidAlgorithm
            | ErrorKind::Base64(_)
            | ErrorKind::Json(_)
            | ErrorKind::Utf8(_)
            | ErrorKind::Crypto(_) => ApiError::from_code(error::BAD_AUTHORIZATION).into(),
            // unexpected error!
            _ => ServerFnError::ServerError(e.to_string()),
        })
}

/// Extracts a token.
///
/// Mostly used for endpoints where a token is required.
#[cfg(feature = "ssr")]
pub async fn extract_token() -> Result<Claims, ServerFnError<ApiError>> {
    use axum::http::header::{self, HeaderMap};
    use cookie::Cookie;
    use leptos_axum::extract;
    use std::str::FromStr;

    let headers = extract::<HeaderMap>()
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    // get cookie
    let cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|cookie| cookie.to_str().ok())
        .filter_map(|cookie| Cookie::from_str(cookie).ok())
        .find(|cookie| cookie.name() == "auth");

    if let Some(auth_cookie) = cookie {
        // TODO check parameters
        decode_token(auth_cookie.value())
    } else {
        // unauthenticated
        Err(ApiError::from_code(error::MISSING_AUTHORIZATION).into())
    }
}

/// Current user infromation.
///
/// May store private information that isn't to be shared otherwise.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentUser {
    /// The username of the user.
    pub username: String,
}

/// Gets information about the current user
#[server(endpoint = "/user/~me")]
pub async fn get_current_user() -> Result<CurrentUser, ServerFnError<ApiError>> {
    use crate::schema::user::get_user;

    let token = extract_token().await?;

    let state = expect_context::<ServerState>();

    let user = get_user(&state.pool, &token.sub)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(user) = user {
        Ok(CurrentUser {
            username: user.username,
        })
    } else {
        Err(ApiError::from_code(error::BAD_AUTHORIZATION).into())
    }
}

/// Shows the currently logged-in user, along with a button to logout.
#[component]
pub fn ShowCurrentUser() -> impl IntoView {
    let (count, set_count) = signal(0);
    let current_user = Resource::new(move || count.get(), |_| get_current_user());

    let show_user = move || match current_user.get() {
        Some(Ok(user)) => Some(Ok(view! { <p>{user.username}</p> })),
        Some(Err(err)) => Some(Err(err)),
        None => None,
    };

    view! {
        <Suspense>
            <ErrorBoundary
                fallback=move |_| view! {
                    <Login on_complete=move || *set_count.write() += 1 />
                }
            >
                {show_user}
            </ErrorBoundary>
        </Suspense>
    }
}
