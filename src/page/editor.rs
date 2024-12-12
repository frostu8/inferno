//! Page editor frontend.

use leptos::{html::Textarea, prelude::*};

use wasm_bindgen::prelude::*;

use web_sys::HtmlTextAreaElement;

use crate::page::edit::PushPageChanges;

/// The page editor.
///
/// # Anti-Footgun
/// Note that this component takes in the content of the page as
/// `initial_content`, which means the content of the page may change, but the
/// initial content will not.
#[component]
pub fn PageEditor(path: Signal<String>, initial_content: String) -> impl IntoView {
    let textarea_ref = NodeRef::<Textarea>::new();

    let push_page_changes = ServerAction::<PushPageChanges>::new();

    Effect::new(move || {
        if let Some(node) = textarea_ref.get() {
            upgrade_editor(node);
        }
    });

    view! {
        <ActionForm attr:class="editor" action=push_page_changes>
            <div class="page-admin">
                <input type="submit" value="Save Changes"/>
            </div>
            <textarea
                node_ref=textarea_ref
                id="page-source"
                name="source"
                rows="40"
            >
                {initial_content}
            </textarea>
            <input type="hidden" name="path" value=path/>
        </ActionForm>
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Inferno, js_name = upgradeEditor)]
    fn upgrade_editor(text_area: HtmlTextAreaElement);
}
