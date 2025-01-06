//! Pages, the top-level construct of inferno.
//!
//! This module contains some shared page codecs and components. All the render
//! logic has been sectioned off in `render` since it is quite complicated.

pub mod edit;
pub mod editor;
pub mod view;

pub use crate::slug::Slug;

use leptos::prelude::*;
use leptos::Params;
use leptos_meta::{Meta, Title};
use leptos_router::{
    components::{Redirect, A},
    hooks::use_params,
    params::Params,
};

use edit::get_page_source;
use editor::PageEditor;
use view::{render_page, RenderedPage};

use crate::components::{Sidebar, SidebarItem};

#[derive(Debug, Params, PartialEq)]
struct PageParams {
    path: Option<String>,
}

/// A component that takes [`PageParams`] as its params and fixes weird looking
/// links.
#[component]
pub fn ValidatePath<F, IV>(inner: F) -> impl IntoView
where
    F: Fn(Signal<Slug>) -> IV + Send + Sync + 'static,
    IV: IntoView + 'static,
{
    let params = use_params::<PageParams>();

    let slug = Memo::new(move |_| {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|s| s.path.as_ref().and_then(|s| Slug::new(s).ok()))
    });
    let slug_unwrap = Signal::derive(move || slug.with(|c| c.as_ref().unwrap().clone()));

    let redirect_path = Signal::derive(move || {
        let params = params.read();
        let path = params
            .as_ref()
            .ok()
            .and_then(|s| s.path.as_deref())
            .unwrap_or("");

        Slug::slugify(path)
            .map(|s| format!("/~/{}", s))
            .unwrap_or_else(|_| "/~/".into())
    });

    view! {
        <Show
            when=move || slug.with(|s| s.is_some())
            fallback=move || {
                let path = redirect_path.get();
                view! { <Redirect path /> }
            }
        >
            {inner(slug_unwrap)}
        </Show>
    }
}

/// Renders a page as it appears on the main screen, using router parameters.
#[component]
pub fn Page() -> impl IntoView {
    view! { <ValidatePath inner=move |path| view! { <PageInner path /> } /> }
}

#[component]
fn PageInner(path: Signal<Slug>) -> impl IntoView {
    use std::path::Path;

    // link to edit the page
    let href_edit_page = Memo::new(move |_| {
        path.with(|path| {
            Path::new("/~edit")
                .join(path.as_str())
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
                <Suspense>{page_suspense}</Suspense>
            </main>
        </div>
    }
}

/// Simply renders a page.
///
/// This will rerender the entire component on a page change. This behavior
/// could change later.
#[component]
pub fn RenderPage(page: RenderedPage) -> impl IntoView {
    // TODO generate summary of page
    let summary = "inferno wiki".to_owned();

    view! {
        <h1 class="title">{page.title.clone()}</h1>
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
    view! { <ValidatePath inner=move |path| view! { <EditPageInner path /> } /> }
}

#[component]
fn EditPageInner(path: Signal<Slug>) -> impl IntoView {
    use std::path::Path;

    // link to go back
    let href_view_page = Memo::new(move |_| {
        path.with(|path| {
            Path::new("/~")
                .join(path.as_str())
                .to_string_lossy()
                .into_owned()
        })
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
                <Suspense>{page_suspense}</Suspense>
            </main>
        </div>
    }
}

/// The page subtitle.
#[component]
pub fn PageSubtitle(path: Signal<Slug>) -> impl IntoView {
    view! {
        <Show when=move || path.with(|p| p.parent().is_some())>
            <h1 class="subtitle">{move || path.with(|p| p.parent().unwrap().to_owned())}</h1>
        </Show>
    }
}
