//! Page rendering markup.

mod html;

pub use html::*;

use std::borrow::Cow;
use std::collections::VecDeque;
use std::iter::FusedIterator;

use pulldown_cmark::{
    CowStr,
    Event::{self, *},
    HeadingLevel, LinkType, Options, Tag, TagEnd,
};

pub const HEADING_ID_PREFIX: &str = "heading-";

/// Creates a Markdown token stream from content.
pub fn parse_markdown(content: &str) -> Parser<pulldown_cmark::Parser> {
    Parser::new(pulldown_cmark::Parser::new_ext(
        content,
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_WIKILINKS
            | Options::ENABLE_SMART_PUNCTUATION,
    ))
}

/// A custom Markdown filter for inferno.
pub struct Parser<'a, I> {
    inner: I,
    heading: Option<HeadingInfo<'a>>,
    buffer: VecDeque<Event<'a>>,
}

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    /// Creates a new `Parser`.
    pub fn new(inner: I) -> Parser<'a, I> {
        Parser {
            inner,
            heading: None,
            buffer: VecDeque::new(),
        }
    }

    fn text(&mut self, text: CowStr<'a>) -> Option<Event<'a>> {
        Some(Event::Text(text))
    }

    fn start_tag(&mut self, tag: Tag<'a>) -> Option<Event<'a>> {
        match tag {
            Tag::Link {
                link_type: LinkType::WikiLink { has_pothole },
                dest_url,
                title,
                id,
            } => Some(Start(Tag::Link {
                link_type: LinkType::WikiLink { has_pothole },
                dest_url: normalize_wikilink(dest_url).into(),
                title,
                id,
            })),
            Tag::Heading {
                id,
                level,
                classes,
                attrs,
            } => {
                self.heading = Some(HeadingInfo {
                    level,
                    id,
                    classes,
                    attrs,
                    events: VecDeque::new(),
                });
                None
            }
            ev => Some(Start(ev)),
        }
    }

    fn end_tag(&mut self, tag: TagEnd) -> Option<Event<'a>> {
        match tag {
            TagEnd::Heading(orig_level) if self.heading.is_some() => {
                // emit heading start
                let HeadingInfo {
                    level,
                    id,
                    classes,
                    attrs,
                    mut events,
                } = self.heading.take().unwrap();

                events.push_front(End(TagEnd::Heading(orig_level)));

                let id = if let Some(id) = id {
                    id
                } else {
                    // generate id
                    let mut text = String::new();

                    for ev in events.iter() {
                        if let Text(str) = ev {
                            text.push_str(str);
                        }
                    }

                    normalize_heading_id(text).into()
                };

                self.buffer = events;

                Some(Start(Tag::Heading {
                    level,
                    id: Some(id),
                    classes,
                    attrs,
                }))
            }
            tag => Some(End(tag)),
        }
    }
}

impl<'a, I> Iterator for Parser<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tos) = self.buffer.pop_back() {
            return Some(tos);
        };

        while let Some(ev) = self.inner.next() {
            let ev = match ev {
                Start(tag) => self.start_tag(tag),
                End(tag) => self.end_tag(tag),
                Text(str) => self.text(str),
                event => Some(event),
            };

            if let Some(ev) = ev {
                if let Some(heading) = self.heading.as_mut() {
                    heading.events.push_front(ev);
                } else {
                    return Some(ev);
                }
            }
        }

        None
    }
}

impl<'a, I> FusedIterator for Parser<'a, I> where I: Iterator<Item = Event<'a>> {}

#[derive(Debug)]
struct HeadingInfo<'a> {
    level: HeadingLevel,
    id: Option<CowStr<'a>>,
    classes: Vec<CowStr<'a>>,
    attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    events: VecDeque<Event<'a>>,
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
    let mut link = link.into();

    if is_uri_absolute(&link) {
        return link;
    }

    // fix fragment here
    if let Some(idx) = link.rfind('#') {
        let normalized = normalize_heading_id(&link[idx + 1..]);
        link = format!("{}#{}", &link[..idx], normalized).into();
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
        } else if in_whitespace && !ch.is_whitespace() {
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
    fn test_normalize_heading_id() {
        assert_eq!(&normalize_heading_id("The Heading"), "the-heading");
        assert_eq!(
            &normalize_heading_id("pepperoni-secret"),
            "pepperoni-secret"
        );

        assert_eq!(&normalize_heading_id("Hi There!"), "hi-there");
    }
}
