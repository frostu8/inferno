import {EditorView} from "codemirror";
import {Extension} from "@codemirror/state"
import {HighlightStyle, syntaxHighlighting} from "@codemirror/language"
import {tags as t} from "@lezer/highlight"

const background = "#001b29",
  cursor = "#FAC7CA",
  highlightBackground = "#001B29",
  lavender = "#A0A0CF",
  awesomeYellow = "#fff3b0",
  rose = "#E87EA1",
  infernoRed = "#c1121f";

const infernoBaseTheme: Extension = EditorView.theme({
  "&": {
    backgroundColor: background
  },
  ".cm-cursor, .cm-dropCursor": {borderLeftColor: cursor},
  ".cm-activeLine": {
    //backgroundColor: "#001F3D",
    backgroundColor: background
  },
  ".cm-gutters": {
    backgroundColor: "#000d14",
    border: "none"
  },
  ".cm-activeLineGutter": {
    backgroundColor: highlightBackground
  }
});

const infernoHighlightStyle = HighlightStyle.define([
  {tag: t.keyword,
   color: rose},
  {tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName],
   color: lavender},
  {tag: [t.function(t.variableName), t.labelName],
   color: "yellow"},
  {tag: [t.color, t.constant(t.name), t.standard(t.name)],
   color: "green"},
  {tag: [t.definition(t.name), t.separator],
   color: "lavender"},
  {tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace],
   color: awesomeYellow},
  {tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)],
   color: "pink"},
  {tag: [t.meta, t.comment],
   color: "cyan"},
  {tag: t.strong,
   fontWeight: "bold"},
  {tag: t.emphasis,
   fontStyle: "italic"},
  {tag: t.strikethrough,
   textDecoration: "line-through"},
  {tag: t.link,
   color: infernoRed,
   textDecoration: "underline"},
  {tag: t.heading,
   fontWeight: "bold",
   textDecoration: "underline",
   color: "white"},
  {tag: [t.atom, t.bool, t.special(t.variableName)],
   color: rose },
  {tag: [t.processingInstruction, t.string, t.inserted],
   color: lavender},
  {tag: t.invalid,
   color: rose},
]);

export const infernoTheme: Extension = [infernoBaseTheme, syntaxHighlighting(infernoHighlightStyle)];
