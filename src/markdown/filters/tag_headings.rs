use std::collections::VecDeque;
use std::iter::FusedIterator;

use super::super::{is_uri_absolute, normalize_heading_id};

use pulldown_cmark::{
    CowStr,
    Event::{self, *},
    HeadingLevel, Tag, TagEnd,
};

/// Markdown filter that adds IDs to headings based on their text content.
pub struct TagHeadings<'a, I> {
    inner: I,
    heading: Option<HeadingInfo<'a>>,
    buffer: VecDeque<Event<'a>>,
}

impl<'a, I> TagHeadings<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    /// Creates a new `TagHeadings`.
    pub fn new(inner: I) -> TagHeadings<'a, I> {
        TagHeadings {
            inner,
            heading: None,
            buffer: VecDeque::new(),
        }
    }
}

impl<'a, I> Iterator for TagHeadings<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tos) = self.buffer.pop_front() {
            return Some(tos);
        };

        for ev in self.inner.by_ref() {
            // transform local links
            let ev = if let Start(Tag::Link {
                link_type,
                mut dest_url,
                title,
                id,
            }) = ev
            {
                if !is_uri_absolute(&dest_url) {
                    if let Some(idx) = dest_url.find('#') {
                        let normalized = normalize_heading_id(&dest_url[idx + 1..]);
                        dest_url = format!("{}#{}", &dest_url[..idx], normalized).into();
                    }
                }

                Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                })
            } else {
                ev
            };

            // filter heading related tags
            match ev {
                Start(Tag::Heading {
                    id,
                    level,
                    classes,
                    attrs,
                }) => {
                    self.heading = Some(HeadingInfo {
                        level,
                        id,
                        classes,
                        attrs,
                        events: Vec::new(),
                    });
                }
                End(TagEnd::Heading(orig_level)) if self.heading.is_some() => {
                    // emit heading start
                    let HeadingInfo {
                        level,
                        id,
                        classes,
                        attrs,
                        mut events,
                    } = self.heading.take().unwrap();

                    events.push(End(TagEnd::Heading(orig_level)));

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

                    self.buffer = events.into();

                    return Some(Start(Tag::Heading {
                        level,
                        id: Some(id),
                        classes,
                        attrs,
                    }));
                }
                ev => {
                    if let Some(heading) = self.heading.as_mut() {
                        heading.events.push(ev);
                    } else {
                        return Some(ev);
                    }
                }
            }
        }

        None
    }
}

impl<'a, I> FusedIterator for TagHeadings<'a, I> where I: Iterator<Item = Event<'a>> {}

#[derive(Debug)]
struct HeadingInfo<'a> {
    level: HeadingLevel,
    id: Option<CowStr<'a>>,
    classes: Vec<CowStr<'a>>,
    attrs: Vec<(CowStr<'a>, Option<CowStr<'a>>)>,
    events: Vec<Event<'a>>,
}
