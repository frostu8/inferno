use std::collections::VecDeque;
use std::iter::FusedIterator;

use pulldown_cmark::{
    CowStr,
    Event::{self, *},
    HeadingLevel, LinkType, Tag, TagEnd,
};

/// Markdown filter that shortens wikilinks.
pub struct ShortenWikiText<'a, I> {
    inner: I,
    link: Option<LinkInfo<'a>>,
    buffer: VecDeque<Event<'a>>,
}

impl<'a, I> ShortenWikiText<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    /// Creates a new `ShortenWikiText`.
    pub fn new(inner: I) -> ShortenWikiText<'a, I> {
        ShortenWikiText {
            inner,
            link: None,
            buffer: VecDeque::new(),
        }
    }
}

impl<'a, I> Iterator for ShortenWikiText<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

struct LinkInfo<'a> {
    link_type: LinkType,
    dest_url: CowStr<'a>,
    title: CowStr<'a>,
    id: CowStr<'a>,
}
