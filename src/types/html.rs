//! Module that defines an [`Html`] tree.

use core::fmt;

use super::tag::Tag;

/// Dom tree structure to represent the parsed html.
///
/// This tree represents the whole parsed HTML. To create an [`Html`] from a
/// string, use the [`crate::parse::parse_html`] function.
///
/// # Examples
///
/// ```
/// use html_parser::prelude::*;
///
/// let _html: Html = parse_html(
///     r#"<nav>
///     <!-- Navigation menu -->
///     <ul>
///         <li href="first">First link</li>
///         <li href="second">Second link</li>
///         <li href="third">Third link</li>
///     </ul>
/// </nav>"#,
/// )
/// .unwrap();
/// ```
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub enum Html {
    /// Comment block
    ///
    /// # Example
    ///
    /// `<!-- some comment -->`
    Comment(String),
    /// Document tag.
    ///
    /// These are tags with exclamation marks
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    #[non_exhaustive]
    Doctype {
        /// Name of the tag
        ///
        /// # Examples
        ///
        /// In the previous example, the name is `doctype`.
        name: String,
        /// Attribute of the tag
        ///
        /// # Examples
        ///
        /// In the previous example, the attribute is `html`.
        attr: Option<String>,
    },
    /// Empty html tree
    ///
    /// Corresponds to an empty string
    #[default]
    Empty,
    /// Tag
    ///
    /// # Examples
    ///
    /// - `<div id="blob">content</div>`
    /// - `<div attr />`
    /// - `</>`
    #[non_exhaustive]
    Tag {
        /// Opening tag
        ///
        /// Contains the name of the tag and its attributes.
        tag: Tag,
        /// Child of the tag
        ///
        /// Everything between the opening and the closing tag.
        ///
        /// # Note
        ///
        /// This is always empty if the tag is self-closing.
        child: Box<Html>,
    },
    /// Raw text
    ///
    /// Text outside of a tag.
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, `a` and `b` are [`Html::Text`] elements
    Text(String),
    /// List of nodes
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, the node is a vector, with [`Html::Text`] `a`,
    /// [`Html::Tag`] `strong` [`Html::Text`] `b`.
    Vec(Box<[Html]>),
}

impl Html {
    /// Checks if an [`Html`] tree is empty
    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Html {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Tag { tag, child } if tag.as_name() == "br" => write!(f, "<br>{child}"),
            Self::Tag { tag, child } => write!(f, "<{tag}>{child}</{}>", tag.as_name()),
            Self::Doctype { name, attr } => match (name, attr) {
                (name_str, Some(attr_str)) => write!(f, "<!{name_str} {attr_str}>"),
                (name_str, None) if name_str.is_empty() => write!(f, "<!>"),
                (name_str, None) => write!(f, "<!{name_str} >"),
            },
            Self::Text(text) => text.fmt(f),
            Self::Vec(vec) => vec.iter().try_for_each(|html| html.fmt(f)),
            Self::Comment(content) => write!(f, "<!--{content}-->"),
        }
    }
}
