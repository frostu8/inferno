//! Page rendering logic.
//!
//! This module is *specifically* the server-side rendering of pages, because
//! it is rather nuanced and long. It also determines some rules on how to
//! parse pages.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use super::SendPage;

use crate::error::Error as ApiError;

/// The main page rendering endpoint.
#[server(endpoint = "/page", input = GetUrl, output = SendPage)]
pub async fn render_page(path: String) -> Result<Option<String>, ServerFnError<ApiError>> {
    use crate::ServerState;
    use ammonia::{Builder, UrlRelative};
    use pulldown_cmark::{html, Parser};
    use std::collections::HashSet;

    let state = expect_context::<ServerState>();

    #[derive(sqlx::FromRow)]
    struct Page {
        pub content: String,
    }

    // get page
    let page = sqlx::query_as::<_, Page>("SELECT content FROM pages WHERE path = $1")
        .bind(&path)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        let content = Parser::new(&page.content);

        let mut html_output = String::with_capacity(page.content.len() * 3 / 2);
        html::push_html(&mut html_output, content);

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

        Ok(Some(html_output))
    } else {
        Ok(None)
    }
}
