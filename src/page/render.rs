//! Page rendering logic.
//!
//! This module is *specifically* the server-side rendering of pages, because
//! it is rather nuanced and long. It also determines some rules on how to
//! parse pages.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use super::Slug;
use crate::error::Error as ApiError;

use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use pulldown_cmark::CowStr;

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
    use ammonia::{Builder, UrlRelative};
    use pulldown_cmark::{html, Event, LinkType, Options, Parser, Tag};
    use std::collections::HashSet;

    let state = expect_context::<ServerState>();

    let token = extract_token().await;

    // get page
    let page = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        let title = path.title();

        let parser = Parser::new_ext(
            &page.content,
            Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_WIKILINKS
                | Options::ENABLE_SMART_PUNCTUATION,
        )
        .map(|event| {
            if let Event::Start(Tag::Link {
                link_type: LinkType::WikiLink,
                dest_url,
                title,
                id,
            }) = event
            {
                Event::Start(Tag::Link {
                    link_type: LinkType::WikiLink,
                    dest_url: normalize_wikilink(dest_url),
                    title,
                    id,
                })
            } else {
                event
            }
        });

        let mut html_output = String::with_capacity(page.content.len() * 3 / 2);
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
            title: title.into(),
            content: html_output,
            edit: token.is_ok(),
        })
    } else {
        Err(ApiError::from_code(error::NOT_FOUND).into())
    }
}

#[cfg(feature = "ssr")]
fn normalize_wikilink(link: CowStr) -> CowStr {
    use regex::RegexBuilder;

    const WIKI_PREFIX: &str = "/~";

    if link.is_empty() {
        return link;
    }

    // check if the link is absolute, if it is, return as is
    // according to RFC 3986; https://www.rfc-editor.org/rfc/rfc3986
    let is_absolute = RegexBuilder::new("^(?:[a-z+\\-.]+:)?//")
        .case_insensitive(true)
        .build()
        .expect("valid regex");

    if is_absolute.is_match(&link) {
        return link;
    }

    let mut result = String::with_capacity(link.len() + 2);
    let mut i = 0;
    let mut mark = 0;
    let mut in_whitespace = false;

    result.push_str(WIKI_PREFIX);

    if !link.starts_with('/') {
        result.push('/');
    }

    while i < link.len() {
        if !in_whitespace && link.as_bytes()[i].is_ascii_whitespace() {
            in_whitespace = true;
            result.push_str(&link[mark..i]);
            result.push('_');
        } else if in_whitespace && !link.as_bytes()[i].is_ascii_whitespace() {
            mark = i;
            in_whitespace = false;
        }

        i += 1;
    }

    result.push_str(&link[mark..]);
    if !link.ends_with('/') {
        result.push('/');
    }
    result.into()
}
