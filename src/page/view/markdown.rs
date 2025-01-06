//! Logic for rendering pages from Markdown to a token stream.

use pulldown_cmark::{CowStr, Event, Event::*, LinkType, Options, Parser, Tag};
use regex::RegexBuilder;

/// Creates a Markdown token stream from content.
pub fn parse(content: &str) -> impl Iterator<Item = Event> {
    Parser::new_ext(
        content,
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_WIKILINKS
            | Options::ENABLE_SMART_PUNCTUATION,
    )
    .map(|event| {
        if let Start(Tag::Link {
            link_type: LinkType::WikiLink,
            dest_url,
            title,
            id,
        }) = event
        {
            Start(Tag::Link {
                link_type: LinkType::WikiLink,
                dest_url: normalize_wikilink(dest_url),
                title,
                id,
            })
        } else {
            event
        }
    })
}

/// Normalizes wikilinks.
pub fn normalize_wikilink(link: CowStr) -> CowStr {
    if is_uri_absolute(&link) {
        return link;
    }

    if link.is_empty() {
        return link;
    }

    let mut result = String::with_capacity(link.len() + 2);
    let mut i = 0;
    let mut mark = 0;
    let mut in_whitespace = false;

    if !link.starts_with('/') {
        result.push('/');
    }

    while i < link.len() {
        if !in_whitespace && link.as_bytes()[i].is_ascii_whitespace() {
            in_whitespace = true;
            result.push_str(&link[mark..i]);
            result.push('_');
        } else if in_whitespace && !link.as_bytes()[i].is_ascii_whitespace() {
            mark = i;
            in_whitespace = false;
        }

        i += 1;
    }

    result.push_str(&link[mark..]);
    if !link.ends_with('/') {
        result.push('/');
    }
    result.into()
}

pub fn is_uri_absolute(uri: &str) -> bool {
    // check if the link is absolute, if it is, return as is
    // according to RFC 3986; https://www.rfc-editor.org/rfc/rfc3986
    RegexBuilder::new("^(?:[a-z][a-z0-9+\\-.]*:)?//")
        .case_insensitive(true)
        .build()
        .expect("valid regex")
        .is_match(uri)
}
