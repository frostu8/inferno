import {EditorView, basicSetup} from "codemirror";
import {EditorState} from "@codemirror/state";
import {markdown} from "@codemirror/lang-markdown";

/**
 * Upgrades a textarea to a CodeMirror editor.
 *
 * What this effectively does is hide the real textarea from the DOM, and
 * inserts the CodeMirror editor in its place. Any updates made to the editor
 * will be cleanly copied.
 *
 * Thanks to how Leptos works internally, this actually makes sense and does
 * not drive a virtual DOM purist insane.
 *
 * @param {HTMLTextAreaElement} textArea - The text area to upgrade.
 */
export function upgradeEditor(textArea: HTMLTextAreaElement) {
  console.log(textArea);

  // hide textarea
  let parent = textArea.parentElement;
  textArea.classList.add("hidden");

  if (parent === null)
    throw new Error("parent is null");

  // create editor plugin
  let inferno = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      // update old textarea
      textArea.textContent = update.state.doc.toString();
    }
  });

  // initialize start state
  let startState = EditorState.create({
    doc: textArea.textContent || "",
    extensions: [basicSetup, markdown(), EditorView.lineWrapping, inferno]
  });

  // create editor
  let editor = new EditorView({state: startState});

  // place in dom
  parent.insertBefore(editor.dom, textArea);
}
