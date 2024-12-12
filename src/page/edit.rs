//! Page editing endpoints and utilities.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use crate::error::Error as ApiError;

use serde::{Deserialize, Serialize};

/// The output of [`get_page_source`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageSource {
    /// The source content of the page.
    pub source: String,
}

impl Default for PageSource {
    fn default() -> Self {
        PageSource { source: "".into() }
    }
}

/// Fetches a page source.
#[server(endpoint = "/page/source", input = GetUrl)]
pub async fn get_page_source(path: String) -> Result<PageSource, ServerFnError<ApiError>> {
    use crate::{account::extract_token, schema::page::get_page_content, ServerState};

    let state = expect_context::<ServerState>();

    // edits MUST be attributed
    let _token = extract_token().await?;

    let content = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(content) = content {
        Ok(PageSource { source: content })
    } else {
        // user is trying to create page
        // send default page source as if it did exist
        Ok(PageSource::default())
    }
}

/// Creates or updates a new page.
#[server(endpoint = "/page/source")]
pub async fn push_page_changes(
    path: String,
    source: String,
) -> Result<(), ServerFnError<ApiError>> {
    use crate::{
        account::extract_token,
        schema::page::{get_page_content, save_change, update_page_content},
        ServerState,
    };
    use diff_match_patch_rs::{Compat, DiffMatchPatch, PatchInput};

    let state = expect_context::<ServerState>();

    // attribute edits on the given token
    let token = extract_token().await?;

    let old_source = get_page_content(&path, &state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .unwrap_or_else(|| String::new());

    if old_source == source {
        // bail early if the two texts are the exact same
        return Ok(());
    }

    // do page diffing
    let dmp = DiffMatchPatch::new();

    let diffs = dmp
        .diff_main::<Compat>(&old_source, &source)
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
    let patches = dmp
        .patch_make(PatchInput::new_diffs(&diffs))
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;
    let changes = dmp.patch_to_text(&patches);

    // save changes
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    // make update to page content
    update_page_content(&path, &source, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    // add change to db
    save_change(&path, &token.sub, &changes, &mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    // commit changes
    tx.commit()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("{:?}", e)))?;

    Ok(())
}
