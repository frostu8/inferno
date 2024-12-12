//! Provides server entrypoints.

use crate::account::UserPanel;
use crate::page::render::Page;

use leptos::prelude::*;
use leptos::Params;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Redirect, Route, Router, Routes},
    hooks::use_params,
    params::Params,
    path, SsrMode,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link href="https://fonts.googleapis.com/css2?family=Noto+Sans:ital,wght@0,100..900;1,100..900&family=Pixelify+Sans:wght@400..700&family=VT323&display=swap" rel="stylesheet" />
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let redirect_to_main = move || {
        view! { <Redirect path="/~/Index"/> }
    };

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/inferno.css"/>

        // sets the document title
        <Title text="inferno"/>

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=path!("") view=redirect_to_main/>
                // weird HACK to get this to not destroy the api or packages
                // but I actually secretly like it in a way, so much that I put
                // it on the sidebar title
                <ParentRoute path=path!("/~") view=Main ssr=SsrMode::Async>
                    <Route path=path!("") view=redirect_to_main/>
                    <Route path=path!("*path") view=GetPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}

/// Component that renders the main content to the side of a sidebar.
///
/// This is the main component useful for almost all pages on the site.
#[component]
pub fn Main() -> impl IntoView {
    view! {
        <div class="view-content">
            <Sidebar/>
            <main>
                <Outlet/>
            </main>
        </div>
    }
}

/// Top level helper component to render a sidebar.
#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <nav id="sidebar">
            <h1>~/inferno</h1>
            <UserPanel/>
        </nav>
    }
}

#[derive(Debug, Params, PartialEq)]
struct GetPageParams {
    path: Option<String>,
}

#[component]
fn GetPage() -> impl IntoView {
    let params = use_params::<GetPageParams>();

    let path = Signal::derive(move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.path.clone())
            .unwrap_or_default()
    });

    view! { <Page path/> }
}
