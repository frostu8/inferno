//! Page rendering markup.

pub mod filters;

use filters::FiltersExt as _;

use std::borrow::Cow;

use pulldown_cmark::{
    Event::{self, *},
    LinkType, Options, Parser, Tag,
};

pub const HEADING_ID_PREFIX: &str = "heading-";

/// Creates a Markdown token stream from content.
pub fn parse(content: &str) -> impl Iterator<Item = Event<'_>> {
    Parser::new_ext(
        content,
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_WIKILINKS
            | Options::ENABLE_SMART_PUNCTUATION,
    )
    .map(|ev| {
        if let Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole },
            dest_url,
            title,
            id,
        }) = ev
        {
            Start(Tag::Link {
                link_type: LinkType::WikiLink { has_pothole },
                dest_url: normalize_wikilink(dest_url).into(),
                title,
                id,
            })
        } else {
            ev
        }
    })
    .tag_headings()
    .shorten_wikitext()
}

/// Checks if a URI is absolute.
pub fn is_uri_absolute(uri: &str) -> bool {
    use regex::RegexBuilder;
    // check if the link is absolute, if it is, return as is
    // according to RFC 3986; https://www.rfc-editor.org/rfc/rfc3986
    RegexBuilder::new("^(?:[a-z][a-z0-9+\\-.]*:)?//")
        .case_insensitive(true)
        .build()
        .expect("valid regex")
        .is_match(uri)
}

/// Normalizes wikilinks.
pub fn normalize_wikilink<'a, T>(link: T) -> Cow<'a, str>
where
    T: Into<Cow<'a, str>>,
{
    let link = link.into();

    if is_uri_absolute(&link) {
        return link;
    }

    if link.starts_with('#') {
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
        } else if in_whitespace && !link.as_bytes()[i].is_ascii_whitespace() {
            result.push('_');
            mark = i;
            in_whitespace = false;
        }

        i += 1;
    }

    if !in_whitespace {
        result.push_str(&link[mark..]);
    }
    if !link.ends_with('/') {
        result.push('/');
    }
    result.into()
}

/// Normalizes heading IDs.
pub fn normalize_heading_id<'a, T>(id: T) -> Cow<'a, str>
where
    T: Into<Cow<'a, str>>,
{
    let id = id.into();

    if id.is_empty() {
        return id;
    }

    let mut result = String::with_capacity(id.len() + HEADING_ID_PREFIX.len());
    result.push_str(HEADING_ID_PREFIX);
    let mut i = 0;
    let mut mark = 0;
    let mut in_whitespace = false;

    while i < id.len() {
        let ch = id[i..].chars().next().unwrap();

        if !in_whitespace && !ch.is_alphanumeric() {
            in_whitespace = true;
            result.push_str(&id[mark..i]);
        } else if in_whitespace && ch.is_alphanumeric() {
            result.push('-');
            mark = i;
            in_whitespace = false;
        }

        if ch.is_ascii_uppercase() {
            result.push_str(&id[mark..i]);
            result.push(ch.to_ascii_lowercase());
            mark = i + ch.len_utf8();
        }

        i += ch.len_utf8();
    }

    if result.is_empty() {
        id
    } else {
        if !in_whitespace {
            result.push_str(&id[mark..]);
        }
        result.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_heading() {
        assert_eq!(&normalize_heading_id("The Heading"), "heading-the-heading");
        assert_eq!(
            &normalize_heading_id("pepperoni-secret"),
            "heading-pepperoni-secret"
        );

        assert_eq!(&normalize_heading_id("Hi There!"), "heading-hi-there");
    }

    #[test]
    fn normalize_heading_with_symbols() {
        assert_eq!(
            &normalize_heading_id("Vanguard \"Natalia\""),
            "heading-vanguard-natalia"
        );
    }
}
