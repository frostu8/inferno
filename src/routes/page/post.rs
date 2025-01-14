//! Page post stuff.

use crate::{
    error::ServerError,
    markdown::{self, is_uri_absolute},
    schema::page::{
        deregister_link, establish_link, get_links_from, get_page_content_for_update, save_change,
        update_page_content,
    },
    slug::Slug,
    universe::CurrentUniverse,
    ServerState,
};

use super::{
    show::{handler as show, QueryParams},
    Context,
};

use std::collections::HashSet;

use axum::extract::{Form, Query, State};
use axum::response::Response;

use eyre::{Report, WrapErr};

use tracing::instrument;

use pulldown_cmark::{Event::*, Tag};

use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct UpdatePage {
    pub source: String,
    #[serde(default)]
    pub latest_change_hash: Option<String>,
}

/// Updates the page source of a page.
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn handler(
    context: Context,
    universe: CurrentUniverse,
    state: State<ServerState>,
    Form(form): Form<UpdatePage>,
) -> Result<Response, ServerError> {
    let Context {
        current_user, path, ..
    } = &context;

    let Ok(current_user) = current_user else {
        // TODO show flash or unauthorized
        return show(context, Query(QueryParams::default()), universe, state).await;
    };

    // Begin transaction for reading things from the db.
    let mut tx = {
        // keep state in this scope to prevent database access
        state
            .pool
            .begin()
            .await
            .wrap_err("failed to start transaction")?
    };

    let old_page = get_page_content_for_update(&mut *tx, universe.locate(path))
        .await
        .wrap_err("failed to get page content")?;

    if let Some(last_change) = old_page.as_ref().map(|c| &c.latest_change_hash) {
        let Some(form_hash) = form.latest_change_hash.as_ref() else {
            // TODO show flash
            return show(context, Query(QueryParams::default()), universe, state).await;
        };

        if last_change != form_hash {
            // TODO show flash
            return show(context, Query(QueryParams::default()), universe, state).await;
        }
    }

    if old_page.as_ref().map(|c| &c.content) == Some(&form.source) {
        // bail early if the two texts are the exact same
        return show(context, Query(QueryParams::default()), universe, state).await;
    }

    let old_source = old_page.as_ref().map(|c| c.content.as_str()).unwrap_or("");

    // do page diffing
    let dmp = DiffMatchPatch::new();

    let diffs = dmp
        .diff_main::<Compat>(old_source, &form.source)
        .map_err(|e| Report::msg(format!("{:?}", e)))?;
    let patches = dmp
        .patch_make(PatchInput::new_diffs(&diffs))
        .map_err(|e| Report::msg(format!("{:?}", e)))?;
    let changes = dmp.patch_to_text(&patches);

    // make update to page content
    update_page_content(&mut *tx, universe.locate(path), &form.source)
        .await
        .wrap_err("failed to update page content")?;

    // get links in source
    let old_links = get_links_from(&mut *tx, universe.locate(path))
        .await
        .wrap_err("failed to get links from page")?
        .into_iter()
        .collect::<HashSet<Slug>>();
    let mut links = HashSet::<Slug>::new();

    for event in markdown::parse(&form.source) {
        if let Start(Tag::Link { dest_url, .. }) = event {
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
            establish_link(&mut *tx, universe.locate(path), link)
                .await
                .wrap_err("failed to establish link")?;
        }
    }

    for link in old_links.iter() {
        // if link is now missing, remove it
        if !links.contains(link) {
            deregister_link(&mut *tx, universe.locate(path), link)
                .await
                .wrap_err("failed to deregister link")?;
        }
    }

    // add change to db
    save_change(
        &mut *tx,
        universe.locate(path),
        &current_user.username,
        &changes,
    )
    .await
    .wrap_err("failed to save changes")?;

    // commit changes
    tx.commit()
        .await
        .wrap_err("failed to commit database changes")?;

    show(context, Query(QueryParams::default()), universe, state).await
}
