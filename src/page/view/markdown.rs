//! Logic for rendering pages from Markdown to a token stream.

use std::collections::VecDeque;
use std::iter::FusedIterator;

use crate::page::{normalize_heading_id, normalize_wikilink};
use pulldown_cmark::{
    CowStr,
    Event::{self, *},
    HeadingLevel, LinkType, Options, Tag, TagEnd,
};

/// Creates a Markdown token stream from content.
pub fn parse(content: &str) -> Parser<pulldown_cmark::Parser> {
    Parser::new(pulldown_cmark::Parser::new_ext(
        content,
        Options::ENABLE_FOOTNOTES
            | Options::ENABLE_TABLES
            | Options::ENABLE_WIKILINKS
            | Options::ENABLE_SMART_PUNCTUATION,
    ))
}

/// A custom Markdown filter.
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
                link_type: LinkType::WikiLink,
                dest_url,
                title,
                id,
            } => Some(Start(Tag::Link {
                link_type: LinkType::WikiLink,
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
