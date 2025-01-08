//! Page-related routes.

use std::collections::HashSet;

use crate::account::{CurrentUser, Error as AccountError};
use crate::html::HtmlTemplate;
use crate::schema::page::{get_existing_links_from, get_page_content};
use crate::slug::Slug;
use crate::{markup, ServerState};

use axum::extract::State;
use axum::{extract::Path, response::Result};

use tracing::instrument;

use askama::Template;

use super::log_error;

#[derive(Template)]
#[template(path = "show.html")]
#[doc(hidden)]
pub struct ShowPageTemplate {
    /// The actual page content.
    page: RenderedPage,
}

struct RenderedPage {
    pub title: String,
    pub content_clean: String,
}

/// Shows a page to the request client.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn show(
    Path(path): Path<Slug>,
    user: Result<CurrentUser, AccountError>,
    state: State<ServerState>,
) -> Result<HtmlTemplate<ShowPageTemplate>> {
    // get page content
    let page = get_page_content(&path, &state.pool)
        .await
        .map_err(log_error)?;

    let links = get_existing_links_from(&path, &state.pool)
        .await
        .map_err(log_error)?
        .into_iter()
        .collect::<HashSet<Slug>>();

    if let Some(page) = page {
        let title = path.title();
        let events = markup::parse_markdown(&page.content);
        let content_clean = markup::to_html(events, links, Some(page.content.len()));

        let page = RenderedPage {
            title: title.into_owned(),
            content_clean,
        };

        Ok(HtmlTemplate::new(ShowPageTemplate { page }))
    } else {
        todo!()
    }
}
