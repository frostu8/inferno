//! `pulldown-cmark` filters included in the inferno binary.

mod shorten_wikitext;
mod tag_headings;

pub use shorten_wikitext::*;
pub use tag_headings::*;

use pulldown_cmark::Event;

/// Exposes filters in a builder chain API.
pub trait FiltersExt<'a>
where
    Self: Iterator<Item = Event<'a>> + Sized,
{
    /// Tags headings.
    fn tag_headings(self) -> TagHeadings<'a, Self> {
        TagHeadings::new(self)
    }

    /// Shortens the display wikitext on WikiLinks when the link hasn't been
    /// potholed.
    fn shorten_wikitext(self) -> ShortenWikiText<'a, Self> {
        ShortenWikiText::new(self)
    }
}

impl<'a, T> FiltersExt<'a> for T where T: Iterator<Item = Event<'a>> + Sized {}
