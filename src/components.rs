//! Shared components.

use crate::account::UserPanel;

use leptos::prelude::*;
use leptos_router::components::{ToHref, A};

/// Top level helper component to render a sidebar.
#[component]
pub fn Sidebar(children: Children) -> impl IntoView {
    view! {
        <nav id="sidebar">
            <h1>~/inferno</h1>
            <UserPanel/>
            {children()}
        </nav>
    }
}

/// A sidebar button.
#[component]
pub fn SidebarItem<T, H>(text: T, href: H) -> impl IntoView
where
    T: Into<String> + Send + Sync + 'static,
    H: ToHref + Send + Sync + 'static,
{
    view! {
        <A attr:class="sidebar-item" href>
            <p>{text.into()}</p>
        </A>
    }
}
