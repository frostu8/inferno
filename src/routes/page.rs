//! Page-related routes.

use std::collections::HashSet;

use crate::account::{CurrentUser, Error as AccountError};
use crate::html::HtmlTemplate;
use crate::schema::page::{
    deregister_link, establish_link, get_existing_links_from, get_links_from, get_page_content,
    get_page_content_for_update, save_change, update_page_content,
};
use crate::slug::Slug;
use crate::universe::CurrentUniverse;
use crate::{
    markup::{self, is_uri_absolute},
    ServerState,
};

use axum::extract::{Form, OriginalUri, Path, State};
use axum::response::{IntoResponse, Redirect, Response, Result};

use pulldown_cmark::{Event, Tag};

use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};

use serde::Deserialize;

use tracing::instrument;

use askama::Template;

use super::log_error;

#[derive(Template)]
#[template(path = "page/show.html")]
struct ShowPageTemplate {
    /// The full URI of the page.
    pub request_path: String,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The actual page content.
    pub page: RenderedPage,
}

#[derive(Template)]
#[template(path = "page/edit.html")]
struct EditPageTemplate {
    /// The full URI of the page.
    pub request_path: String,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The data about the page.
    pub page: Page,
}

#[derive(Template)]
#[template(path = "page/404.html")]
struct ShowNotFound {
    /// The full URI of the page.
    pub request_path: String,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
}

struct RenderedPage {
    pub content_clean: String,
}

struct Page {
    pub content: String,
    pub latest_change_hash: Option<String>,
}

/// Shows a page to the request client.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn show(
    OriginalUri(uri): OriginalUri,
    Path(path): Path<Slug>,
    universe: Option<CurrentUniverse>,
    current_user: Result<CurrentUser, AccountError>,
    state: State<ServerState>,
) -> Result<Response> {
    let universe_id = universe.as_ref().map(|u| u.id);

    // get page content
    let page = get_page_content(&state.pool, universe_id, &path)
        .await
        .map_err(log_error)?;

    let links = get_existing_links_from(&state.pool, universe_id, &path)
        .await
        .map_err(log_error)?
        .into_iter()
        .collect::<HashSet<Slug>>();

    if let Some(page) = page {
        let events = markup::parse_markdown(&page.content);
        let content_clean = markup::to_html(events, links, Some(page.content.len()));

        let page = RenderedPage { content_clean };

        Ok(HtmlTemplate::new(ShowPageTemplate {
            request_path: uri.path().to_string(),
            current_user: current_user.ok(),
            path,
            page,
        })
        .into_response())
    } else {
        Ok(HtmlTemplate::new(ShowNotFound {
            request_path: uri.path().to_string(),
            current_user: current_user.ok(),
            path,
        })
        .into_response())
    }
}

/// Shows the edit page to the request client.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn edit(
    OriginalUri(uri): OriginalUri,
    Path(path): Path<Slug>,
    universe: Option<CurrentUniverse>,
    current_user: Result<CurrentUser, AccountError>,
    state: State<ServerState>,
) -> Result<Response> {
    let universe_id = universe.as_ref().map(|u| u.id);

    let Ok(current_user) = current_user else {
        // TODO show flash
        return Ok(Redirect::to(&format!("/~/{}", path)).into_response());
    };

    // get page content
    let page = get_page_content(&state.pool, universe_id, &path)
        .await
        .map_err(log_error)?;

    if let Some(crate::schema::page::Page {
        content,
        latest_change_hash,
    }) = page
    {
        Ok(HtmlTemplate::new(EditPageTemplate {
            request_path: uri.path().to_string(),
            current_user: Some(current_user),
            path,
            page: Page {
                content,
                latest_change_hash: Some(latest_change_hash),
            },
        })
        .into_response())
    } else {
        Ok(HtmlTemplate::new(EditPageTemplate {
            request_path: uri.path().to_string(),
            current_user: Some(current_user),
            path,
            page: Page {
                content: String::new(),
                latest_change_hash: None,
            },
        })
        .into_response())
    }
}

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct UpdatePageSource {
    pub source: String,
    #[serde(default)]
    pub latest_change_hash: Option<String>,
}

/// Updates the page source of a page.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn post(
    OriginalUri(uri): OriginalUri,
    Path(path): Path<Slug>,
    universe: Option<CurrentUniverse>,
    current_user: Result<CurrentUser, AccountError>,
    state: State<ServerState>,
    Form(form): Form<UpdatePageSource>,
) -> Result<Response> {
    let universe_id = universe.as_ref().map(|u| u.id);

    let Ok(current_user) = current_user else {
        // TODO show flash or unauthorized
        return Ok(Redirect::to(&format!("/~/{}", path)).into_response());
    };

    // Begin transaction for reading things from the db.
    let mut tx = {
        // keep state in this scope to prevent database access
        state.pool.begin().await.map_err(log_error)?
    };

    let old_page = get_page_content_for_update(&mut *tx, universe_id, &path)
        .await
        .map_err(log_error)?;

    if let Some(last_change) = old_page.as_ref().map(|c| &c.latest_change_hash) {
        let Some(form_hash) = form.latest_change_hash.as_ref() else {
            // TODO show flash
            return Ok(Redirect::to(&format!("/~/{}", path)).into_response());
        };

        if last_change != form_hash {
            // TODO show flash
            return Ok(Redirect::to(&format!("/~/{}", path)).into_response());
        }
    }

    if old_page.as_ref().map(|c| &c.content) == Some(&form.source) {
        // bail early if the two texts are the exact same
        return show(
            OriginalUri(uri),
            Path(path),
            universe,
            Ok(current_user),
            state,
        )
        .await;
    }

    let old_source = old_page.as_ref().map(|c| c.content.as_str()).unwrap_or("");

    // do page diffing
    let dmp = DiffMatchPatch::new();

    let diffs = dmp
        .diff_main::<Compat>(old_source, &form.source)
        .map_err(|e| format!("{:?}", e))
        .map_err(log_error)?;
    let patches = dmp
        .patch_make(PatchInput::new_diffs(&diffs))
        .map_err(|e| format!("{:?}", e))
        .map_err(log_error)?;
    let changes = dmp.patch_to_text(&patches);

    // make update to page content
    update_page_content(&mut *tx, universe_id, &path, &form.source)
        .await
        .map_err(log_error)?;

    // get links in source
    let old_links = get_links_from(&mut *tx, universe_id, &path)
        .await
        .map_err(log_error)?
        .into_iter()
        .collect::<HashSet<Slug>>();
    let mut links = HashSet::<Slug>::new();

    for event in markup::parse_markdown(&form.source) {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            if !is_uri_absolute(&dest_url) {
                // wikilinks are normalized by markdown::parse, so
                // un-normalize them here
                let dest_url = dest_url
                    .find('#')
                    .map(|idx| &dest_url[..idx])
                    .unwrap_or(&dest_url);
                let dest_url = dest_url.trim_matches('/');
                if let Ok(slug) = Slug::new(dest_url) {
                    links.insert(slug);
                }
            }
        }
    }

    for link in links.iter() {
        // if link is missing, add it
        if !old_links.contains(link) {
            establish_link(&mut *tx, universe_id, &path, link)
                .await
                .map_err(log_error)?;
        }
    }

    for link in old_links.iter() {
        // if link is now missing, remove it
        if !links.contains(link) {
            deregister_link(&mut *tx, universe_id, &path, link)
                .await
                .map_err(log_error)?;
        }
    }

    // add change to db
    save_change(
        &mut *tx,
        universe_id,
        &path,
        &current_user.username,
        &changes,
    )
    .await
    .map_err(log_error)?;

    // commit changes
    tx.commit().await.map_err(log_error)?;

    show(
        OriginalUri(uri),
        Path(path),
        universe,
        Ok(current_user),
        state,
    )
    .await
}

mod filters {
    use crate::slug::Slug;

    pub fn title(slug: &Slug) -> ::askama::Result<String> {
        Ok(slug.title().into_owned())
    }
}
