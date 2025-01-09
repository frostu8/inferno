//! Page-related routes.

use std::collections::HashSet;

use crate::account::{CurrentUser, Error as AccountError};
use crate::html::HtmlTemplate;
use crate::schema::page::{get_existing_links_from, get_page_content};
use crate::slug::Slug;
use crate::{markup, ServerState};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response, Result},
};

use tracing::instrument;

use askama::Template;

use super::log_error;

#[derive(Template)]
#[template(path = "page/show.html")]
struct ShowPageTemplate {
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The actual page content.
    pub page: RenderedPage,
}

#[derive(Template)]
#[template(path = "page/not_found.html")]
struct ShowNotFound {
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
}

struct RenderedPage {
    pub content_clean: String,
}

/// Shows a page to the request client.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn show(
    Path(path): Path<Slug>,
    current_user: Result<CurrentUser, AccountError>,
    state: State<ServerState>,
) -> Result<Response> {
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
        let events = markup::parse_markdown(&page.content);
        let content_clean = markup::to_html(events, links, Some(page.content.len()));

        let page = RenderedPage { content_clean };

        Ok(HtmlTemplate::new(ShowPageTemplate {
            current_user: current_user.ok(),
            path,
            page,
        })
        .into_response())
    } else {
        Ok(HtmlTemplate::new(ShowNotFound {
            current_user: current_user.ok(),
            path,
        })
        .into_response())
    }
}

mod filters {
    use crate::slug::Slug;

    pub fn title(slug: &Slug) -> ::askama::Result<String> {
        Ok(slug.title().into_owned())
    }
}
