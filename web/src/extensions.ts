import {MarkdownConfig} from "@lezer/markdown";
import {tags as t} from "@lezer/highlight";

const WikilinkDelimiter = {resolve: "Wikilink", mark: "WikilinkMark"};

export const Wikilinks: MarkdownConfig = {
  defineNodes: [{
    name: "Wikilink",
    style: {"Wikilink/...": t.link}
  }, {
    name: "WikilinkMark",
    style: t.processingInstruction
  }],
  parseInline: [{
    name: "Wikilink",
    parse(cx, next, pos) {
      if (next == 91 /* '[' */ && cx.char(pos + 1) == 91) {
        return cx.addDelimiter(WikilinkDelimiter, pos, pos + 2, true, false);
      } else if (next == 93 /* ']' */ && cx.char(pos + 1) == 93) {
        return cx.addDelimiter(WikilinkDelimiter, pos, pos + 2, false, true);
      } else {
        return -1;
      }
    },
    before: "Link"
  }]
}

