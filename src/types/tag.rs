//! Module to define the tag data structure.

use core::fmt;
use core::hash::Hash;
use core::mem::take;

use crate::errors::safe_unreachable;

/// Name and optionally a value for an attribute of a tag.
///
/// Attributes provide information about a tag. They can consist in a simple
/// name, or also have a value, after an `=` sign. The values are always
/// surrounded either by single or double quotes.
#[allow(
    clippy::allow_attributes,
    clippy::derived_hash_with_manual_eq,
    reason = "hash on enum doesn't depend of variant data"
)]
#[derive(Debug, Hash)]
#[non_exhaustive]
pub(crate) enum Attribute {
    /// Name of the attribute, when it doesn't have a value
    ///
    /// # Examples
    ///
    /// In `<button />`, the name of the attribute is `button`.
    NameNoValue(PrefixName),
    /// Name of the attribute
    ///
    /// # Examples
    ///
    /// `<div id="blob"/>`
    #[non_exhaustive]
    NameValue {
        /// Whether double or single quotes were used to define the value
        ///
        /// Equals `true` if the attribute value was delimited by double quotes,
        /// and false otherwise.
        double_quote: bool,
        /// Name of the attribute
        ///
        /// # Examples
        ///
        /// In `<div id="blob" />`, the name of the first attribute is `id`.
        ///
        /// # Note
        ///
        /// Attribute names can have prefixes, like in `<a xlink:href="link"/>`
        name: PrefixName,
        /// Value of the attribute
        ///
        /// # Examples
        ///
        /// - In `<div id="blob" />`, the value of the first attribute is
        ///   `"blob"`.
        value: String,
    },
}

impl Attribute {
    /// Converts an existent [`Attribute::NameNoValue`] to a
    /// [`Attribute::NameValue`].
    ///
    /// # Panics
    ///
    /// If called on a [`Attribute::NameValue`]
    #[coverage(off)]
    pub(crate) fn add_value(&mut self, double_quote: bool) {
        if let Self::NameNoValue(name) = self {
            *self = Self::NameValue { double_quote, name: take(name), value: String::new() }
        } else {
            safe_unreachable("Never create attribute value twice from parser.")
        }
    }

    /// Returns the name of an attribute
    pub const fn as_name(&self) -> &PrefixName {
        match self {
            Self::NameNoValue(prefix_name) => prefix_name,
            Self::NameValue { name, .. } => name,
        }
    }

    /// Returns the value of an attribute
    pub const fn as_value(&self) -> Option<&String> {
        match self {
            Self::NameNoValue(_) => None,
            Self::NameValue { value, .. } => Some(value),
        }
    }

    /// Returns the value of an attribute
    fn into_value(self) -> Option<String> {
        match self {
            Self::NameNoValue(_) => None,
            Self::NameValue { value, .. } => Some(value),
        }
    }

    /// Pushes a character into the value of the [`PrefixName`]
    #[coverage(off)]
    pub(crate) fn push_value(&mut self, ch: char) {
        if let Self::NameValue { value, .. } = self {
            value.push(ch);
        } else {
            safe_unreachable("Never push to attribute before creation.")
        }
    }
}

impl From<PrefixName> for Attribute {
    #[inline]
    fn from(name: PrefixName) -> Self {
        Self::NameNoValue(name)
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Attribute {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameNoValue(prefix_name) => write!(f, " {prefix_name}"),
            Self::NameValue { double_quote, name, value } => write!(f, " {name}").and_then(|()| {
                let del = if *double_quote { '"' } else { '\'' };
                write!(f, "={del}{value}{del}")
            }),
        }
    }
}

/// [`Tag`] name with optionally a prefix.
///
/// The prefix of a tag name is the part before the colon.
///
/// # Examples
///
/// - In `<a:b id="blob"/>`, the prefix is `a` and the name is `b`.
/// - In `<a id="blob"/>`, the name is `a` and there is no prefix.
#[non_exhaustive]
#[derive(PartialEq, Eq, Debug, Hash)]
pub(crate) enum PrefixName {
    /// Name of the fragment
    ///
    /// No prefix here, i.e., no colon found.
    Name(String),
    /// Prefix and name of the fragment
    Prefix(String, String),
}

impl PrefixName {
    /// Pushes a character into a [`PrefixName`]
    pub(crate) fn push_char(&mut self, ch: char) {
        match self {
            Self::Name(name) | Self::Prefix(_, name) => name.push(ch),
        }
    }

    /// Pushes a colon into a [`PrefixName`]
    ///
    /// This informs us that there was a prefix.
    ///
    /// # Errors
    ///
    /// Returns an error if there is already a prefix, i.e., if a colon as
    /// already been found.
    pub(crate) fn push_colon(&mut self) -> Result<(), &'static str> {
        *self = match self {
            Self::Name(name) => Self::Prefix(take(name), String::new()),
            Self::Prefix(..) => return Err("Found 2 colons ':' in attribute name."),
        };
        Ok(())
    }
}

impl Default for PrefixName {
    #[inline]
    fn default() -> Self {
        Self::Name(String::new())
    }
}

impl<T: Into<String>> From<T> for PrefixName {
    #[inline]
    fn from(value: T) -> Self {
        let value_str: String = value.into();
        if value_str.contains(':') {
            let mut prefix = String::new();
            let mut iter = value_str.chars();
            while let Some(ch) = iter.next() {
                if ch == ':' {
                    break; // end of prefix
                }
                prefix.push(ch);
            }
            Self::Prefix(prefix, iter.collect())
        } else {
            Self::Name(value_str)
        }
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for PrefixName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => name.fmt(f),
            Self::Prefix(prefix, name) => write!(f, "{prefix}:{name}"),
        }
    }
}

/// Tag structure, with its name and attributes
///
/// # Examples
///
/// ```
/// use html_parser::prelude::*;
///
/// let html = parse_html("<a enabled href='https://crates.io'>").unwrap();
/// if let Html::Tag { tag, .. } = html {
///     assert!(tag.as_name() == "a");
///     assert!(tag.find_attr_value("enabled").is_none());
///     assert!(
///         tag.find_attr_value("href")
///             .is_some_and(|value| value == "https://crates.io")
///     );
///     let value: String = tag.into_attr_value("href").unwrap();
///     assert!(&value == "https://crates.io");
/// } else {
///     unreachable!();
/// }
/// ```
#[expect(
    clippy::field_scoped_visibility_modifiers,
    reason = "use methods for API but visibility needed by parser"
)]
#[derive(Default, Debug)]
pub struct Tag {
    /// Attributes of the tag. See [`Attribute`].
    pub(crate) attrs: Vec<Attribute>,
    /// Name of the tag.
    ///
    /// # Examples
    ///
    /// - `<div id="blob">` as name `div`
    /// - `<>` as an empty name
    pub(crate) name: String,
}

impl Tag {
    /// Returns the name of the tag
    ///
    /// # Examples
    ///
    /// ```
    /// use html_parser::prelude::*;
    ///
    /// let html = parse_html("<div />").unwrap();
    /// if let Html::Tag { tag, .. } = html {
    ///     assert!(tag.as_name() == "div");
    /// } else {
    ///     unreachable!();
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn as_name(&self) -> &String {
        &self.name
    }

    /// Finds the value of the attribute of the given name
    ///
    /// # Returns
    ///
    /// - `Some(value)` if `name = value` is present in the [`Tag`]
    /// - `None` if the attribute doesn't exist, or if it doesn't have a value
    ///
    /// # Examples
    ///
    /// ```
    /// use html_parser::prelude::*;
    ///
    /// let html = parse_html(r#"<a id="std doc" enabled xlink:href="https://std.rs"/>"#).unwrap();
    ///
    /// if let Html::Tag { tag, .. } = html {
    ///     assert!(tag.find_attr_value("enabled").is_none());
    ///     assert!(
    ///         tag.find_attr_value("xlink:href")
    ///             .map(|value| value.as_ref())
    ///             == Some("https://std.rs")
    ///     );
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn find_attr_value<T: Into<String>>(&self, name: T) -> Option<&String> {
        let prefix_name = PrefixName::from(name);
        self.attrs
            .iter()
            .find(|attr| attr.as_name() == &prefix_name)
            .and_then(|attr| attr.as_value())
    }

    /// Finds the value of the attribute of the given name
    ///
    /// # Returns
    ///
    /// - `Some(value)` if `name = value` is present in the [`Tag`]
    /// - `None` if the attribute doesn't exist, or if it doesn't have a value
    ///
    /// # Examples
    ///
    /// ```
    /// use html_parser::prelude::*;
    ///
    /// let html = parse_html(r#"<a enabled/>"#).unwrap();
    ///
    /// if let Html::Tag { tag, .. } = html {
    ///     assert!(tag.into_attr_value("enabled").is_none());
    /// } else {
    ///     unreachable!()
    /// }
    ///
    /// let html = parse_html(r#"<a id="std doc" href="https://std.rs"/>"#).unwrap();
    ///
    /// if let Html::Tag { tag, .. } = html {
    ///     assert!(
    ///         tag.into_attr_value("href")
    ///             .is_some_and(|value| &value == "https://std.rs")
    ///     );
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn into_attr_value<T: Into<String>>(self, name: T) -> Option<String> {
        let prefix_name = PrefixName::from(name);
        self.attrs
            .into_iter()
            .find(|attr| attr.as_name() == &prefix_name)?
            .into_value()
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        self.attrs.iter().try_for_each(|attr| attr.fmt(f))
    }
}

/// Builder returns by the parser when run on a tag.
pub(crate) enum TagBuilder {
    /// Closing tag
    ///
    /// # Examples
    ///
    /// `</,>` and `</div>`
    Close(String),
    /// Document tag
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    Document {
        /// Name of the document tag.
        ///
        /// # Examples
        ///
        /// From the example above, the name is `doctype`.
        name: String,
        /// Attribute of the document tag.
        ///
        /// # Examples
        ///
        /// From the example above, the name is `html`.
        attr: Option<String>,
    },
    /// Opening tag
    ///
    /// Doesn't a `/` at the end of the tag declaration.
    ///
    /// # Examples
    ///
    /// `<div>` and `<>` and `<div id="blob" enabled>`
    Open(Tag),
    /// Self-closing tag.
    ///
    /// Contains a `/` at the end of the tag declaration.
    ///
    /// # Examples
    ///
    /// `<p />` and `<div id="blob" enabled />`
    OpenClose(Tag),
    /// Opening block comment
    ///
    /// # Examples
    ///
    /// `<!--`
    OpenComment,
}

/// Closing type of the tag.
#[derive(Debug)]
pub(crate) enum TagType {
    /// Closed tag
    ///
    /// This means the closing part of the tag was found.
    ///
    /// # Examples
    ///
    /// `</div>` was read after `<div>`
    Closed,
    /// Opened tag
    ///
    /// This means the closing part of the tag was not yet found.
    ///
    /// # Examples
    ///
    /// `<div>` was read, but not the associated `</div>` yet.
    Opened,
    /// Self-closing tag
    ///
    /// This means tag closes itself, with a '/' character.
    ///
    /// # Examples
    ///
    /// `<div id="blob" />` and `</>`
    SelfClosing,
}

impl TagType {
    /// Checks if tag is still open.
    ///
    /// This happens when the tag is not self closing, and the corresponding
    /// closing tag has not yet been found.
    ///
    /// # Examples
    ///
    /// This happens when a <div> was read, but </div> was not yet read.
    pub(super) const fn is_open(&self) -> bool {
        matches!(self, Self::Opened)
    }
}
