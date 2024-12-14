//! Slug and path operations.

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{
    de::{self, Deserialize},
    Serialize,
};

/// A slug.
///
/// A slug is a path of a wiki page fit to display in URLs comfortably. Pages
/// can have names with spaces in them, so this helps convert between the two.
///
/// ## Whitespace
/// On page creation, spaces in a page's name are converted into underscores.
/// When an author makes a page with multiple spaces in its name, it is a
/// visibility and accessibility concern, and is more often than not a mistake.
///
/// The slug cannot also begin or end with underscores.
///
/// ## Leading/Trailing slash
/// It is okay for the slug to have a leading or trailing slash, but it will be
/// treated the same as without one. That is to say;
///
/// ```
/// # use inferno::slug::Slug;
/// let slug1 = Slug::new("/Index/");
/// let slug2 = Slug::new("Index");
///
/// assert_eq!(slug1, slug2);
/// ```
///
/// This type gaurantees any string held within it does not contain any
/// whitespace, and that it isn't empty.
#[repr(transparent)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    /// Creates a new slug from a string.
    ///
    /// This operation fails if any part of the slug has whitespace, the slug
    /// is empty, or consecutive underscores appear.
    pub fn new<'a>(s: impl Into<Cow<'a, str>>) -> Result<Slug, ConvertError> {
        let s = s.into();

        if s.is_empty() {
            return Err(ConvertError {
                position: 0,
                kind: ConvertErrorKind::Empty,
            });
        }

        if s.starts_with('_') {
            return Err(ConvertError {
                position: 0,
                kind: ConvertErrorKind::InvalidChar('_'),
            });
        }

        if s.ends_with('_') {
            return Err(ConvertError {
                position: s.len() - 1,
                kind: ConvertErrorKind::InvalidChar('_'),
            });
        }

        // check string
        let mut prev_ch = None;
        for (pos, ch) in s.chars().enumerate() {
            if is_valid_char(ch) {
                if ch == '_' && prev_ch == Some('_') {
                    return Err(ConvertError {
                        position: pos,
                        kind: ConvertErrorKind::InvalidChar(ch),
                    });
                }
            } else {
                return Err(ConvertError {
                    position: pos,
                    kind: ConvertErrorKind::InvalidChar(ch),
                });
            }

            prev_ch = Some(ch);
        }

        Ok(Slug(s.into_owned()))
    }

    /// Converts a plain string into a `Slug` by converting it via simple
    /// rules.
    ///
    /// This can still produce [`ConvertErrorKind::Empty`] if the input or
    /// output string is empty.
    pub fn slugify(input: impl AsRef<str>) -> Result<Slug, ConvertError> {
        let input = input.as_ref();

        // trim text and split on invalid chars
        let out = input
            .trim()
            .split(|c| !is_valid_char(c))
            .fold(String::new(), |mut acc, x| {
                if acc.is_empty() {
                    x.into()
                } else {
                    acc.push('_');
                    acc.push_str(x);
                    acc
                }
            });

        Slug::new(out)
    }

    /// Joins two slugs together by a '/'.
    pub fn join(&self, other: &Slug) -> Slug {
        Slug::new(format!("{}/{}", self.as_str(), other.as_str())).unwrap()
    }

    /// Returns a substring of the slug that indicates its parent directory.
    ///
    /// Returns `None` if it is on the base directory. The result is also
    /// slug-compatible and can be converted/unwrapped.
    pub fn parent(&self) -> Option<&str> {
        self.as_str().rfind('/').map(|idx| &self.as_str()[..idx])
    }

    /// Gets the title of a slug by getting the last segment and replacing
    /// underscores with ASCII spaces.
    pub fn title<'a>(&'a self) -> Cow<'a, str> {
        let title = match self.as_str().rfind('/') {
            Some(idx) => &self.as_str()[idx + 1..],
            None => self.as_str(),
        };

        let mut result = String::new();
        let mut i = 0;
        let mut mark = 0;

        while i < title.len() {
            let ch = title[i..].chars().next().unwrap();
            if ch == '_' {
                result.push_str(&title[mark..i]);
                result.push(' ');
                mark = i + 1;
            }
            i += 1;
        }

        if mark == 0 {
            title.into()
        } else {
            result.push_str(&title[mark..]);
            result.into()
        }
    }

    /// Gets a reference to the str inside.
    pub fn as_str(&self) -> &str {
        match (self.0.starts_with('/'), self.0.ends_with('/')) {
            (false, false) => &self.0,
            (true, false) => &self.0[1..],
            (false, true) => &self.0[..(self.0.len() - 1)],
            (true, true) => &self.0[1..(self.0.len() - 1)],
        }
    }

    /// Gets a reference to the str inside, including any leading slash that
    /// might be in there.
    pub fn as_str_raw(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Slug> for String {
    fn from(value: Slug) -> Self {
        value.0
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

impl Serialize for Slug {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Slug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Slug::new(s).map_err(|e| de::Error::custom(e))
    }
}

impl FromStr for Slug {
    type Err = ConvertError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Slug::new(s)
    }
}

/// Error for converting to a slug.
#[derive(Debug, PartialEq)]
pub struct ConvertError {
    position: usize,
    kind: ConvertErrorKind,
}

#[derive(Debug, PartialEq)]
pub enum ConvertErrorKind {
    InvalidChar(char),
    Empty,
}

impl Display for ConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ConvertErrorKind::InvalidChar(ch) => {
                write!(f, "invalid char '{}' @ col {}", ch, self.position + 1)
            }
            ConvertErrorKind::Empty => write!(f, "slug is empty"),
        }
    }
}

impl std::error::Error for ConvertError {}

fn is_valid_char(ch: char) -> bool {
    !ch.is_whitespace()
}

#[cfg(test)]
mod tests {
    use super::Slug;

    #[test]
    fn test_slugify_1() {
        let input = "Index For Winners";
        let output = Slug::new("Index_For_Winners").unwrap();

        let slug = Slug::slugify(&input);

        assert_eq!(slug, Ok(output));
    }
}
