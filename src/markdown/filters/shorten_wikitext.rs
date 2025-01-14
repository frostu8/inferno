use std::collections::VecDeque;
use std::iter::FusedIterator;

use pulldown_cmark::{
    CowStr,
    Event::{self, *},
    LinkType, Tag, TagEnd,
};

/// Markdown filter that shortens wikilinks.
pub struct ShortenWikiText<'a, I> {
    inner: I,
    link: Option<WikiLinkInfo<'a>>,
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
        if let Some(tos) = self.buffer.pop_front() {
            return Some(tos);
        }

        for ev in self.inner.by_ref() {
            match ev {
                Start(Tag::Link {
                    link_type: LinkType::WikiLink { has_pothole: false },
                    dest_url,
                    title,
                    id,
                }) => {
                    self.link = Some(WikiLinkInfo {
                        dest_url,
                        title,
                        id,
                        events: Vec::new(),
                    });
                }
                End(TagEnd::Link) if self.link.is_some() => {
                    // emit link start
                    let WikiLinkInfo {
                        dest_url,
                        title,
                        id,
                        events,
                    } = self.link.take().unwrap();

                    // build text
                    let text = events
                        .into_iter()
                        .filter_map(|ev| match ev {
                            Text(text) => Some(text),
                            _ => None,
                        })
                        .fold(String::new(), |mut acc, s| {
                            acc.push_str(&s);
                            acc
                        });

                    // run shortening algorithm
                    let shortened = shorten_wikitext(&text);

                    self.buffer.push_back(Text(shortened.to_owned().into()));
                    self.buffer.push_back(End(TagEnd::Link));

                    return Some(Start(Tag::Link {
                        link_type: LinkType::WikiLink { has_pothole: false },
                        dest_url,
                        title,
                        id,
                    }));
                }
                ev => {
                    if let Some(link) = self.link.as_mut() {
                        link.events.push(ev);
                    } else {
                        return Some(ev);
                    }
                }
            }
        }

        None
    }
}

impl<'a, I> FusedIterator for ShortenWikiText<'a, I> where I: Iterator<Item = Event<'a>> {}

struct WikiLinkInfo<'a> {
    dest_url: CowStr<'a>,
    title: CowStr<'a>,
    id: CowStr<'a>,
    events: Vec<Event<'a>>,
}

fn shorten_wikitext(text: &str) -> &str {
    if let Some(text) = text.strip_prefix('#') {
        return text;
    }

    let text = text.find('#').map(|idx| &text[..idx]).unwrap_or(text);
    let text = text
        .strip_prefix('/')
        .unwrap_or(text)
        .strip_suffix('/')
        .unwrap_or(text);
    text.rfind('/').map(|idx| &text[idx + 1..]).unwrap_or(text)
}
