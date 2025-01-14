use std::{collections::HashSet, iter::FusedIterator};

use crate::{markdown::is_uri_absolute, slug::Slug};

use pulldown_cmark::{
    Event::{self, *},
    Tag, TagEnd,
};
use pulldown_cmark_escape::{escape_href, escape_html};

/// Decorates links, adding classes and marking external links.
///
/// ## Warning
/// This consumes any [`Tag::Link`] events and replaces them with HTML events.
/// This should be the last link transformer in the chain of filters.
pub struct DecorateLinks<I> {
    inner: I,
    resolved_links: Option<HashSet<Slug>>,
}

impl<I> DecorateLinks<I> {
    /// Creates a new `DecorateLinks`.
    pub fn new(inner: I) -> DecorateLinks<I> {
        DecorateLinks {
            inner,
            resolved_links: None,
        }
    }

    /// Tells the decorator to use a list of resolved links, calculated in a
    /// previous pass.
    pub fn with_resolved_links(self, resolved_links: HashSet<Slug>) -> DecorateLinks<I> {
        DecorateLinks {
            resolved_links: Some(resolved_links),
            ..self
        }
    }
}

impl<'a, I> Iterator for DecorateLinks<I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(ev) = self.inner.next() else {
            return None;
        };

        let ev = match ev {
            Start(Tag::Link {
                link_type: _,
                dest_url,
                title,
                // unused
                id: _,
            }) => {
                let mut html = String::new();
                let is_absolute = is_uri_absolute(&dest_url);

                html.push_str("<a href=\"");

                if !is_absolute && dest_url.starts_with('/') {
                    html.push_str("/~");
                }

                escape_href(&mut html, &dest_url).unwrap();
                if !title.is_empty() {
                    html.push_str("\" title=\"");
                    escape_html(&mut html, &title).unwrap();
                }
                html.push_str("\"");

                let mut classes = AnchorClassWriter::new(&mut html);

                if let Some(resolved_links) = self.resolved_links.as_ref() {
                    let slug = dest_url
                        .find('#')
                        .map(|idx| &dest_url[..idx])
                        .unwrap_or(&dest_url);
                    let slug = slug.trim_matches('/');

                    if !is_absolute
                        // link could be a interlink fragment
                        && !dest_url.starts_with('#')
                        && Slug::new(slug)
                            .map(|s| !resolved_links.contains(&s))
                            .unwrap_or(true)
                    {
                        classes.add("noexist");
                    }
                }

                if is_absolute {
                    classes.add("external-link");
                }

                classes.finish();

                html.push_str(">");

                Html(html.into())
            }
            End(TagEnd::Link) => Html("</a>".into()),
            ev => ev,
        };

        Some(ev)
    }
}

impl<'a, I> FusedIterator for DecorateLinks<I> where I: Iterator<Item = Event<'a>> + FusedIterator {}

struct AnchorClassWriter<'a> {
    out: &'a mut String,
    classes: usize,
}

impl<'a> AnchorClassWriter<'a> {
    pub fn new(out: &'a mut String) -> AnchorClassWriter<'a> {
        AnchorClassWriter { out, classes: 0 }
    }

    pub fn add(&mut self, class: &str) {
        if self.classes > 0 {
            self.out.push_str(" ");
        } else {
            self.out.push_str(" class=\"");
        }

        self.classes += 1;

        self.out.push_str(class);
    }

    pub fn finish(self) {
        if self.classes > 0 {
            self.out.push_str("\"");
        }
    }
}
