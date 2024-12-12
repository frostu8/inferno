//! Page editor frontend.

use leptos::prelude::*;

use crate::page::edit::PushPageChanges;

/// The page editor.
///
/// # Anti-Footgun
/// Note that this component takes in the content of the page as
/// `initial_content`, which means the content of the page may change, but the
/// initial content will not.
#[component]
pub fn PageEditor(path: Signal<String>, initial_content: String) -> impl IntoView {
    let push_page_changes = ServerAction::<PushPageChanges>::new();

    view! {
        <ActionForm attr:class="editor" action=push_page_changes>
            <div class="page-admin">
                <input type="submit" value="Save Changes"/>
            </div>
            <textarea id="page-source" name="source" rows="40">
                {initial_content}
            </textarea>
            <input type="hidden" name="path" value=path/>
        </ActionForm>
    }
}
