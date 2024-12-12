//! Page rendering logic.
//!
//! This module is *specifically* the server-side rendering of pages, because
//! it is rather nuanced and long. It also determines some rules on how to
//! parse pages.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use crate::error::Error as ApiError;

use serde::{Deserialize, Serialize};

/// The output of [`render_page`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderedPage {
    /// The content of the page, to be rendered as-is and sanitized.
    pub content: String,
    /// Whether the user can edit it or not.
    pub edit: bool,
}

/// The main page rendering endpoint.
#[server(endpoint = "/page", input = GetUrl)]
pub async fn render_page(path: String) -> Result<RenderedPage, ServerFnError<ApiError>> {
    use crate::{account::extract_token, error, schema::page::get_page_content, ServerState};
    use ammonia::{Builder, UrlRelative};
    use pulldown_cmark::{html, Parser};
    use std::collections::HashSet;

    let state = expect_context::<ServerState>();

    let token = extract_token().await;

    // get page
    let content = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(content) = content {
        let parser = Parser::new(&content);

        let mut html_output = String::with_capacity(content.len() * 3 / 2);
        html::push_html(&mut html_output, parser);

        // sanitize html
        // sorry sir, I won't be taking any XSS anytime soon
        //
        // cleans after Markdown to prevent any nasty expansion tricks
        let mut generic_attributes = HashSet::new();
        generic_attributes.insert("class");

        let html_output = Builder::default()
            .generic_attributes(generic_attributes)
            .link_rel(Some("noopener noreferrer"))
            .url_relative(UrlRelative::PassThrough)
            .clean(&html_output)
            .to_string();

        // construct page info
        Ok(RenderedPage {
            content: html_output,
            edit: token.is_ok(),
        })
    } else {
        Err(ApiError::from_code(error::NOT_FOUND).into())
    }
}
