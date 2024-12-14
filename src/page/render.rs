//! Page rendering logic.
//!
//! This module is *specifically* the server-side rendering of pages, because
//! it is rather nuanced and long. It also determines some rules on how to
//! parse pages.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

#[cfg(feature = "ssr")]
use ammonia::UrlRelativeEvaluate;
#[cfg(feature = "ssr")]
use std::borrow::Cow;
#[cfg(feature = "ssr")]
use std::collections::HashMap;

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
    use crate::{
        account::extract_token,
        error,
        schema::page::{expand_wikilink, get_page_content},
        ServerState,
    };
    use ammonia::{Builder, UrlRelative};
    use pulldown_cmark::{html, Event, Options, Parser, Tag};
    use std::collections::HashSet;
    use url::Url;

    const WIKI_PREFIX: &str = "/~/";

    let state = expect_context::<ServerState>();

    let token = extract_token().await;

    // get page
    let page = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        let title = path.title();

        let mut slugs_to_resolve = HashSet::new();

        let parser = Parser::new_ext(
            &page.content,
            Options::ENABLE_FOOTNOTES
                | Options::ENABLE_TABLES
                | Options::ENABLE_WIKILINKS
                | Options::ENABLE_SMART_PUNCTUATION,
        )
        .inspect(|event| {
            if let Event::Start(Tag::Link { dest_url, .. }) = event {
                // HACK kinda silly way to figure out if a url is relative
                if Url::parse(dest_url).is_err() {
                    if let Ok(slug) = Slug::new(dest_url.clone()) {
                        slugs_to_resolve.insert(slug);
                    }
                }
            }
        });

        let mut html_output = String::with_capacity(page.content.len() * 3 / 2);
        html::push_html(&mut html_output, parser);

        // resolve links
        let mut resolved_map = UrlResolver::new(WIKI_PREFIX);

        for slug in slugs_to_resolve {
            let key = slug.as_str_raw().to_owned();

            let resolved = if let Some(slug) = expand_wikilink(&slug, &state.pool)
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?
            {
                slug
            } else {
                // could not expand, so default behavior
                if let Some(parent) = path.parent() {
                    // TODO: maybe the unwrap should be the responsibility of
                    // the callee
                    Slug::new(parent).unwrap().join(&slug)
                } else {
                    slug
                }
            };

            resolved_map.insert(key, resolved);
        }

        // sanitize html
        // sorry sir, I won't be taking any XSS anytime soon
        //
        // cleans after Markdown to prevent any nasty expansion tricks
        let mut generic_attributes = HashSet::new();
        generic_attributes.insert("class");

        let html_output = Builder::default()
            .generic_attributes(generic_attributes)
            .link_rel(Some("noopener noreferrer"))
            .url_relative(UrlRelative::Custom(Box::new(resolved_map)))
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
struct UrlResolver<'a> {
    prefix: &'a str,
    resolved_map: HashMap<String, String>,
}

#[cfg(feature = "ssr")]
impl<'a> UrlResolver<'a> {
    fn new(prefix: &'a str) -> UrlResolver<'a> {
        UrlResolver {
            prefix,
            resolved_map: HashMap::new(),
        }
    }

    fn insert(&mut self, key: String, slug: Slug) {
        self.resolved_map
            .insert(key, format!("{}{}", self.prefix, slug.as_str()));
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.resolved_map.get(key).map(|s| s.as_str())
    }
}

#[cfg(feature = "ssr")]
impl<'a> UrlRelativeEvaluate<'a> for UrlResolver<'a> {
    fn evaluate<'url>(&self, url: &'url str) -> Option<Cow<'url, str>> {
        Some(
            self.get(url)
                .map(|c| Cow::Owned(c.to_owned()))
                .unwrap_or_else(|| Cow::Borrowed(url)),
        )
    }
}
