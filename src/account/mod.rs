//! Account authentication, management and creation.

pub mod login;
pub mod logout;

use logout::LogoutUser;

use crate::components::SidebarItem;
use crate::user::{get_current_user, CurrentUser};
#[cfg(feature = "ssr")]
use crate::{
    error::{self, Error as ApiError},
    ServerState,
};

#[cfg(feature = "ssr")]
use serde::{Deserialize, Serialize};

use leptos::prelude::*;
use leptos_router::hooks::use_location;

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

/// Shows the currently logged-in user, along with a button to logout.
#[component]
pub fn UserDigest(user: CurrentUser) -> impl IntoView {
    let logout_user = ServerAction::<LogoutUser>::new();

    view! {
        <ActionForm attr:class="user-digest" action=logout_user>
            <p>"Signed in as " <span class="username">{user.username}</span></p>
            // HACK this is the only way to tell the router to refresh the page
            <input type="hidden" name="redirect_to" value="/" />
            <input class="sidebar-item" type="submit" value="Logout" />
        </ActionForm>
    }
}

/// The sidebar session panel. Shows [`UserDigest`] if the user is not logged
/// in.
#[component]
pub fn SidebarSession() -> impl IntoView {
    let current_user = OnceResource::new(get_current_user());

    let location = use_location();

    let login_url = move || {
        location.pathname.with(|p| {
            format!(
                "/~account/login?redirect_to={}",
                url_escape::encode_query(p)
            )
        })
    };

    let fallback = move || view! { <SidebarItem href=login_url text="Login" /> };

    view! {
        <Suspense fallback>
            {move || Suspend::new(async move {
                let current_user = current_user.await;
                match current_user {
                    Ok(user) => {

                        view! { <UserDigest user /> }
                            .into_any()
                    }
                    Err(_) => fallback().into_any(),
                }
            })}
        </Suspense>
    }
}
