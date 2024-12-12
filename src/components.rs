//! Shared components.

use crate::account::UserPanel;

use leptos::prelude::*;

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
