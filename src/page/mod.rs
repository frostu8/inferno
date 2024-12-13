//! Pages, the top-level construct of inferno.
//!
//! This module contains some shared page codecs and components. All the render
//! logic has been sectioned off in `render` since it is quite complicated.

pub mod edit;
pub mod editor;
pub mod render;

use leptos::prelude::*;
use leptos::Params;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_params, params::Params};

use edit::get_page_source;
use editor::PageEditor;
use render::{render_page, RenderedPage};

use crate::components::{Sidebar, SidebarItem};

#[derive(Debug, Params, PartialEq)]
struct PageParams {
    path: Option<String>,
}

/// Renders a page as it appears on the main screen, using router parameters.
#[component]
pub fn Page() -> impl IntoView {
    use std::path::Path;

    let params = use_params::<PageParams>();

    let path = Signal::derive(move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.path.clone())
            .unwrap_or_default()
    });

    // link to edit the page
    let href_edit_page = Memo::new(move |_| {
        path.with(|path| {
            Path::new("/~edit")
                .join(path)
                .to_string_lossy()
                .into_owned()
        })
    });

    // wait for content
    let page = Resource::new(move || path.get(), render_page);
    let page_suspense = move || {
        Suspend::new(async move {
            match page.await {
                Ok(page) => view! { <RenderPage page /> }.into_any(),
                Err(ServerFnError::WrappedServerError(e)) if e.not_found() => view! {
                    // TODO only show edit button to logged users
                    <p>
                        "This page does not exist. You can create it "
                        <A href=href_edit_page>"here"</A> "."
                    </p>
                }
                .into_any(),
                // TODO better 500 pages
                Err(_) => view! { "error" }.into_any(),
            }
        })
    };
    let page_editable = move || {
        page.with(|page| match page {
            Some(Ok(page)) => page.edit,
            _ => false,
        })
    };

    view! {
        <div class="view-content">
            <Sidebar>
                // edit page button
                <Suspense>
                    <Show when=page_editable>
                        <SidebarItem text="Edit Page" href=href_edit_page />
                    </Show>
                </Suspense>
            </Sidebar>
            <main>
                <PageSubtitle path />
                <h1 class="title">
                    {move || {
                        path.with(|path| {
                            Path::new(path).file_name().map(|s| s.to_string_lossy().into_owned())
                        })
                    }}
                </h1>
                <Suspense>{page_suspense}</Suspense>
            </main>
        </div>
    }
}

/// Simply renders a page.
#[component]
pub fn RenderPage(page: RenderedPage) -> impl IntoView {
    // TODO generate summary of page
    let summary = "inferno wiki".to_owned();

    view! {
        <div class="page-content" inner_html=page.content></div>
        // meta controls
        <Title text=page.title.clone() />

        <Meta name="description" content=summary />
        <Meta name="og:title" content=page.title.clone() />
        <Meta name="og:type" content="article" />
    }
}

/// Provides an interface for editing a page.
#[component]
pub fn EditPage() -> impl IntoView {
    use std::path::Path;

    let params = use_params::<PageParams>();

    let path = Signal::derive(move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.path.clone())
            .unwrap_or_default()
    });

    // link to go back
    let href_view_page = Memo::new(move |_| {
        path.with(|path| Path::new("/~").join(path).to_string_lossy().into_owned())
    });

    // wait for content
    let page = Resource::new(move || path.get(), get_page_source);
    let page_suspense = move || {
        Suspend::new(async move {
            match page.await {
                Ok(page) => view! { <PageEditor path page /> }.into_any(),
                // TODO better 500 pages
                Err(_) => view! { "error" }.into_any(),
            }
        })
    };

    view! {
        <div class="view-content">
            <Sidebar>
                // return to normal page
                <SidebarItem text="View Page" href=href_view_page />
            </Sidebar>
            <main>
                <PageSubtitle path />
                <h1 class="title">
                    "Editing "
                    {move || {
                        path.with(|path| {
                            Path::new(path).file_name().map(|s| s.to_string_lossy().into_owned())
                        })
                    }}
                </h1>
                <Suspense>{page_suspense}</Suspense>
            </main>
        </div>
    }
}

/// The page subtitle.
#[component]
pub fn PageSubtitle(path: Signal<String>) -> impl IntoView {
    use std::path::Path;

    view! {
        <h1 class="subtitle">
            {move || {
                path.with(|path| {
                    Path::new(path)
                        .parent()
                        .map(|s| s.to_string_lossy())
                        .and_then(|s| {
                            if !s.is_empty() { Some(format!("{}/", s)) } else { None }
                        })
                })
            }}
        </h1>
    }
}
