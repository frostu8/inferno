//! User access, permission management.

use serde::{Deserialize, Serialize};

use crate::error::Error as ApiError;

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

/// Current user infromation.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentUser {
    /// The username of the user.
    pub username: String,
}

/// Gets information about the current user
#[server(endpoint = "/user/~me", input = GetUrl)]
pub async fn get_current_user() -> Result<CurrentUser, ServerFnError<ApiError>> {
    use crate::account::extract_token;
    use crate::schema::user::get_user;
    use crate::{error, ServerState};

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
