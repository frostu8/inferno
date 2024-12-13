//! Page editor frontend.

use leptos::{html::Textarea, prelude::*};

use wasm_bindgen::prelude::*;

use web_sys::HtmlTextAreaElement;

use crate::page::edit::{PageSource, PushPageChanges};

/// The page editor.
///
/// # Anti-Footgun
/// Note that this component takes in the content of the page as
/// `initial_content`, which means the content of the page may change, but the
/// initial content will not.
#[component]
pub fn PageEditor(path: Signal<String>, page: PageSource) -> impl IntoView {
    let textarea_ref = NodeRef::<Textarea>::new();

    let (last_hash, set_last_hash) = signal(page.latest_change_hash);

    let push_page_changes = ServerAction::<PushPageChanges>::new();

    let err_msg = move || {
        let result = push_page_changes.value().get();

        match result {
            Some(Err(err)) => err.to_string(),
            _ => unreachable!(),
        }
    };

    // CodeMirror support, calls into the Inferno ext JS.
    Effect::new(move || {
        if let Some(node) = textarea_ref.get() {
            upgrade_editor(node);
        }
    });

    // For making updates.
    Effect::new(move || {
        if let Some(Ok(change)) = push_page_changes.value().get() {
            set_last_hash(change.hash);
        }
    });

    view! {
        <ActionForm attr:class="editor" action=push_page_changes>
            // TODO: error modals
            <Show when=move || push_page_changes.value().with(|c| matches!(c, Some(Err(_))))>
                <p>{err_msg}</p>
            </Show>
            <div class="page-admin">
                <input type="submit" value="Save Changes" />
            </div>
            <textarea node_ref=textarea_ref id="page-source" name="source" rows="40">
                {page.source}
            </textarea>
            <input type="hidden" name="latest_change_hash" value=last_hash />
            <input type="hidden" name="path" value=path />
        </ActionForm>
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Inferno, js_name = upgradeEditor)]
    fn upgrade_editor(text_area: HtmlTextAreaElement);
}
