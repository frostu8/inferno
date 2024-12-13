//! Provides server entrypoints.

use crate::page::{EditPage, Page};

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    path, SsrMode,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link
                    href="https://fonts.googleapis.com/css2?family=Noto+Sans:ital,wght@0,100..900;1,100..900&family=Pixelify+Sans:wght@400..700&family=VT323&display=swap"
                    rel="stylesheet"
                />
                <script src="/inferno.ext.js" async></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let redirect_to_main = move || {
        view! { <Redirect path="/~/Index" /> }
    };

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/inferno.css" />

        // sets the document title
        <Title text="inferno" />

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                // accessibility
                <Route path=path!("") view=redirect_to_main ssr=SsrMode::Async />
                <Route path=path!("~") view=redirect_to_main ssr=SsrMode::Async />
                // ~ prefix is a weird HACK to get this to not destroy the api
                // or packages but I actually secretly like it in a way, so
                // much that I put it on the sidebar title
                <Route path=path!("~/*path") view=Page ssr=SsrMode::Async />
                // same for page editing
                <Route path=path!("~edit/*path") view=EditPage ssr=SsrMode::Async />
            </Routes>
        </Router>
    }
}
