//! Page viewing.
//!
//! This module is *specifically* the server-side rendering of pages, because
//! it is rather nuanced and long. It also determines some rules on how to
//! parse pages.

#[cfg(feature = "ssr")]
mod render;

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use super::Slug;
use crate::error::Error as ApiError;

use serde::{Deserialize, Serialize};

/// The output of [`render_page`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderedPage {
    /// The title of the page.
    ///
    /// This can already be derived from the path on the client side, but is
    /// here in case the functionality changes.
    pub title: String,
    /// The content of the page, to be rendered as-is and sanitized.
    pub content: String,
    /// Whether the user can edit it or not.
    pub edit: bool,
}

/// The main page rendering endpoint.
#[server(endpoint = "/page", input = GetUrl)]
pub async fn render_page(path: Slug) -> Result<RenderedPage, ServerFnError<ApiError>> {
    use crate::{account::extract_token, error, schema::page::get_page_content, ServerState};

    let state = expect_context::<ServerState>();

    let token = extract_token().await;

    // get page
    let page = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        let title = path.title();
        let html_output = render::to_html(&page.content);

        Ok(RenderedPage {
            title: title.into(),
            content: html_output,
            // TODO edit permissions
            edit: token.is_ok(),
        })
    } else {
        Err(ApiError::from_code(error::NOT_FOUND).into())
    }
}
