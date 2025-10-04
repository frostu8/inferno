//! Page routes.
//!
//! These routes make up for most of the actual Wiki-things on inferno.

#[doc(hidden)]
pub mod show;
pub use show::handler as show;

#[doc(hidden)]
pub mod post;
pub use post::handler as post;

use std::collections::HashSet;
use std::fmt::{self, Debug, Formatter};

use http::{request::Parts, uri::Uri};

use axum::{
    extract::{FromRef, FromRequestParts, Path},
    response::{IntoResponse, Redirect, Response},
    RequestPartsExt,
};

use sqlx::Executor;

use eyre::{Report, WrapErr};

use crate::{
    account::{CurrentUser, Error as AccountError},
    error::ServerError,
    markdown::{self, filters::FiltersExt as _},
    schema::{
        page::{get_existing_links_from, get_page_content, Page},
        Database as PreferredDatabase,
    },
    slug::Slug,
    ServerState,
};

use ammonia::{Builder, UrlRelative};

/// A [`Page`] renderer.
pub struct Renderer<'a> {
    page: &'a Page,
    resolved_links: Option<HashSet<Slug>>,
}

impl<'a> Renderer<'a> {
    /// Creates a new `Renderer`.
    pub fn new(page: &'a Page) -> Renderer<'a> {
        Renderer {
            page,
            resolved_links: None,
        }
    }

    /// Fetches the links for the page.
    pub async fn resolve_links<'c, E>(self, db: E) -> Result<Self, sqlx::Error>
    where
        E: Executor<'c, Database = PreferredDatabase>,
    {
        let links = get_existing_links_from(db, &self.page.slug)
            .await?
            .into_iter()
            .collect::<HashSet<Slug>>();

        Ok(Renderer {
            resolved_links: Some(links),
            ..self
        })
    }

    /// Renders the page.
    pub fn render(mut self) -> RenderedPage {
        let mut events = markdown::parse(&self.page.content).decorate_links();

        if let Some(resolved_links) = self.resolved_links.take() {
            events = events.with_resolved_links(resolved_links);
        }

        // render markdown
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events);

        // sterilize resulting html
        let mut generic_attributes = HashSet::new();
        generic_attributes.insert("class");
        generic_attributes.insert("id");

        let out = Builder::default()
            .generic_attributes(generic_attributes)
            .attribute_filter(|element, attribute, value| match (element, attribute) {
                ("h1", "id")
                | ("h2", "id")
                | ("h3", "id")
                | ("h4", "id")
                | ("h5", "id")
                | ("h6", "id")
                    if value.starts_with("heading-") =>
                {
                    Some(value.into())
                }
                // deny all ids on any other element
                (_, "id") => None,
                _ => Some(value.into()),
            })
            .link_rel(Some("noopener noreferrer"))
            .url_relative(UrlRelative::PassThrough)
            .clean(&html_output)
            .to_string();

        RenderedPage { content: out }
    }
}

/// A rendered page.
#[derive(Debug)]
pub struct RenderedPage {
    content: String,
}

impl RenderedPage {
    /// Starts the rendering of a page.
    pub fn build(page: &Page) -> Renderer<'_> {
        Renderer::new(page)
    }

    /// The rendered content.
    pub fn rendered(&self) -> &str {
        &self.content
    }
}

/// Common request details between all requests in these routes.
pub struct Context {
    /// The request URI.
    pub request_uri: Uri,
    /// The user of the request.
    pub current_user: Result<CurrentUser, AccountError>,
    /// The path of the page.
    pub path: Slug,
    /// The sidebar page, if there is one.
    pub sidebar: Option<RenderedPage>,
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("request_uri", &self.request_uri)
            .field("current_user", &self.current_user)
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl<S> FromRequestParts<S> for Context
where
    ServerState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = BadSlugRedirect<ServerError>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request_uri = parts.uri.clone();
        let current_user = parts.extract_with_state(state).await;
        let Path(path) = parts
            .extract::<Path<String>>()
            .await
            .expect("valid path arg");

        let path = path
            .parse::<Slug>()
            .map_err(|_| BadSlugRedirect::BadSlug(path))?;

        // render sidebar
        let state = ServerState::from_ref(state);
        let sidebar = render_sidebar(&state).await?;

        Ok(Context {
            request_uri,
            current_user,
            path,
            sidebar,
        })
    }
}

async fn render_sidebar(state: &ServerState) -> Result<Option<RenderedPage>, ServerError> {
    let state = ServerState::from_ref(state);
    let sidebar_page = get_page_content(&state.pool, &Slug::new("Special:Sidebar").unwrap())
        .await
        .wrap_err("failed to get Special:Sidebar page")?;

    Ok(sidebar_page.map(|page| RenderedPage::build(&page).render()))
}

/// Redirects a bad slug.
#[derive(Debug)]
pub enum BadSlugRedirect<E> {
    BadSlug(String),
    Other(E),
}

impl<E> IntoResponse for BadSlugRedirect<E>
where
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            BadSlugRedirect::BadSlug(slug) => {
                // slugify and redirect
                match Slug::slugify(slug) {
                    Ok(slug) => Redirect::permanent(&format!("/~/{}", slug)).into_response(),
                    // resulting slug is empty
                    Err(_) => Redirect::to("/~/Index").into_response(),
                }
            }
            BadSlugRedirect::Other(res) => res.into_response(),
        }
    }
}

impl From<Report> for BadSlugRedirect<ServerError> {
    fn from(value: Report) -> Self {
        BadSlugRedirect::Other(ServerError::from(value))
    }
}

impl<E> From<E> for BadSlugRedirect<E> {
    fn from(value: E) -> Self {
        BadSlugRedirect::Other(value)
    }
}

/// Askama filters.
mod filters {
    use crate::slug::Slug;

    #[allow(dead_code)]
    pub fn title(slug: &Slug) -> ::askama::Result<String> {
        Ok(slug.title().into_owned())
    }
}
