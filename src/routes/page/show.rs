//! Page show routes.

use crate::{
    account::CurrentUser,
    error::ServerError,
    html::HtmlTemplate,
    schema::page::{get_page_content, Page},
    slug::Slug,
    universe::CurrentUniverse,
    ServerState,
};

use super::{filters, Context, RenderedPage};

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};

use tracing::instrument;

use serde::Deserialize;

use eyre::WrapErr;

use http::Uri;

use askama::Template;

/// Query parameters for the page show routes.
#[derive(Debug, Default, Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub action: PageAction,
}

/// A page action. One of `edit` or `view`.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageAction {
    #[default]
    View,
    ViewSource,
    Edit,
}

#[derive(Template)]
#[template(path = "page/show.html")]
pub struct ShowPageTemplate {
    /// The full URI of the page.
    pub request_uri: Uri,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The sidebar page content, if there is one.
    pub sidebar: Option<RenderedPage>,
    /// The actual page content.
    pub page: RenderedPage,
}

#[derive(Template)]
#[template(path = "page/edit.html")]
pub struct EditPageTemplate {
    /// The full URI of the page.
    pub request_uri: Uri,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The sidebar page content, if there is one.
    pub sidebar: Option<RenderedPage>,
    /// Whether the page is being viewed in read-only mode.
    pub read_only: bool,
    /// The page.
    ///
    /// Since the page may not exist yet, this is a different type.
    pub page: MaybePage,
}

#[derive(Template)]
#[template(path = "page/404.html")]
pub struct NotFoundTemplate {
    /// The full URI of the page.
    pub request_uri: Uri,
    /// The user.
    pub current_user: Option<CurrentUser>,
    /// The path of the page.
    pub path: Slug,
    /// The sidebar page content, if there is one.
    pub sidebar: Option<RenderedPage>,
}

pub struct MaybePage {
    pub content: String,
    pub latest_change_hash: Option<String>,
}

impl From<Page> for MaybePage {
    fn from(value: Page) -> Self {
        MaybePage {
            content: value.content,
            latest_change_hash: Some(value.latest_change_hash),
        }
    }
}

/// Shows a page to the request client.
///
/// This also shows the edit page
#[instrument]
#[cfg_attr(debug_assertions, axum::debug_handler)]
pub async fn handler(
    context: Context,
    params: Query<QueryParams>,
    universe: CurrentUniverse,
    state: State<ServerState>,
) -> Result<Response, ServerError> {
    let Context {
        request_uri,
        current_user,
        path,
        sidebar,
    } = context;

    match params.action {
        PageAction::View => {
            // get page content
            let page = get_page_content(&state.pool, universe.locate(&path))
                .await
                .wrap_err("failed to get page content")?;

            if let Some(page) = page {
                let page = RenderedPage::build(&page)
                    .resolve_links(&state.pool, &universe)
                    .await
                    .wrap_err("failed to resolve links")?
                    .render();

                Ok(HtmlTemplate::new(ShowPageTemplate {
                    request_uri,
                    current_user: current_user.ok(),
                    path,
                    sidebar,
                    page,
                })
                .into_response())
            } else {
                Ok(HtmlTemplate::new(NotFoundTemplate {
                    request_uri,
                    current_user: current_user.ok(),
                    path,
                    sidebar,
                })
                .into_response())
            }
        }
        PageAction::Edit | PageAction::ViewSource => {
            let read_only =
                matches!(params.action, PageAction::ViewSource) || current_user.is_err();

            // get page content
            let page = get_page_content(&state.pool, universe.locate(&path))
                .await
                .wrap_err("failed to get page content")?;

            if let Some(crate::schema::page::Page {
                content,
                latest_change_hash,
                ..
            }) = page
            {
                Ok(HtmlTemplate::new(EditPageTemplate {
                    request_uri,
                    current_user: current_user.ok(),
                    path,
                    sidebar,
                    read_only,
                    page: MaybePage {
                        content,
                        latest_change_hash: Some(latest_change_hash),
                    },
                })
                .into_response())
            } else {
                Ok(HtmlTemplate::new(EditPageTemplate {
                    request_uri,
                    current_user: current_user.ok(),
                    path,
                    sidebar,
                    read_only,
                    page: MaybePage {
                        content: String::new(),
                        latest_change_hash: None,
                    },
                })
                .into_response())
            }
        }
    }
}
