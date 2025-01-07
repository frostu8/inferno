//! Page editing endpoints and utilities.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use super::Slug;
use crate::error::Error as ApiError;

use serde::{Deserialize, Serialize};

/// The output of [`get_page_source`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageSource {
    /// The title of the page.
    ///
    /// See [`Page::title`].
    ///
    /// [`Page::title`]: crate::page::render::Page
    pub title: String,
    /// The source content of the page.
    pub source: String,
    /// The latest change of the hash.
    ///
    /// Used to protect against concurrent accesses. In the future, a better
    /// system might be implemented.
    pub latest_change_hash: Option<String>,
}

impl Default for PageSource {
    fn default() -> Self {
        PageSource {
            title: "".into(),
            source: "".into(),
            latest_change_hash: None,
        }
    }
}

/// Fetches a page source.
#[server(endpoint = "/page/source", input = GetUrl)]
pub async fn get_page_source(path: Slug) -> Result<PageSource, ServerFnError<ApiError>> {
    use crate::{account::extract_token, schema::page::get_page_content, ServerState};

    let state = expect_context::<ServerState>();

    let title = path.title().into();

    // edits MUST be attributed
    let _token = extract_token().await?;

    let page = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        Ok(PageSource {
            title,
            source: page.content,
            latest_change_hash: Some(page.latest_change_hash),
        })
    } else {
        // user is trying to create page
        // send default page source as if it did exist
        Ok(PageSource {
            title,
            ..PageSource::default()
        })
    }
}

/// Result for [`push_page_changes`]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChangeResult {
    /// The hash of the change.
    ///
    /// May be `None` if the page did not change.
    pub hash: Option<String>,
}

/// Creates or updates a new page.
#[server(endpoint = "/page/source")]
pub async fn push_page_changes(
    path: Slug,
    latest_change_hash: String,
    source: String,
) -> Result<ChangeResult, ServerFnError<ApiError>> {
    use super::{is_uri_absolute, view::markdown};
    use crate::{
        account::extract_token,
        error,
        schema::page::{
            deregister_link, establish_link, get_links_from, get_page_content_for_update,
            save_change, update_page_content,
        },
        ServerState,
    };
    use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};
    use pulldown_cmark::{Event, Tag};
    use std::collections::HashSet;

    // attribute edits on the given token
    let token = extract_token().await?;

    // Begin transaction for reading things from the db.
    let mut tx = {
        // keep state in this scope to prevent database access
        let state = expect_context::<ServerState>();
        state
            .pool
            .begin()
            .await
            .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?
    };

    let old_page = get_page_content_for_update(&path, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(last_change) = old_page.as_ref().map(|c| &c.latest_change_hash) {
        if *last_change != latest_change_hash {
            return Err(ApiError::from_code(error::PAGE_ALREADY_CHANGED).into());
        }
    }

    if old_page.as_ref().map(|c| &c.content) == Some(&source) {
        // bail early if the two texts are the exact same
        return Ok(ChangeResult { hash: None });
    }

    let old_source = old_page.as_ref().map(|c| c.content.as_str()).unwrap_or("");

    // do page diffing
    let dmp = DiffMatchPatch::new();

    let diffs = dmp
        .diff_main::<Compat>(old_source, &source)
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
    let patches = dmp
        .patch_make(PatchInput::new_diffs(&diffs))
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
    let changes = dmp.patch_to_text(&patches);

    // make update to page content
    update_page_content(&path, &source, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    // get links in source
    let old_links = get_links_from(&path, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?
        .into_iter()
        .collect::<HashSet<Slug>>();
    let mut links = HashSet::<Slug>::new();

    for event in markdown::parse(&source) {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            if !is_uri_absolute(&dest_url) {
                // wikilinks are normalized by markdown::parse, so
                // un-normalize them here
                if let Ok(slug) = Slug::new(dest_url.trim_matches('/')) {
                    links.insert(slug);
                }
            }
        }
    }

    for link in links.iter() {
        // if link is missing, add it
        if !old_links.contains(link) {
            establish_link(&path, link, &mut *tx)
                .await
                .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
        }
    }

    for link in old_links.iter() {
        // if link is now missing, remove it
        if !links.contains(link) {
            deregister_link(&path, link, &mut *tx)
                .await
                .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
        }
    }

    // add change to db
    let hash = save_change(&path, &token.sub, &changes, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    // commit changes
    tx.commit()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    Ok(ChangeResult { hash: Some(hash) })
}
