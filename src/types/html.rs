//! Module that defines an [`Html`] tree.

use core::{fmt, mem::take};

use crate::safe_unreachable;

use super::tag::{Tag, TagClosingStatus, TagType};

/// Dom tree structure to represent the parsed html.
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum Html {
    /// Comment block
    ///
    /// # Example
    ///
    /// `<!-- some comment -->`
    Comment {
        /// Content of the comment
        ///
        /// # Examples
        ///
        /// In the previous example, the content is `some content`.
        content: String,
        /// Fullness of the comment
        ///
        /// `full` is `true` iff the closing `-->` was found for this comment.
        ///
        /// # Examples
        ///
        /// In the previous example, the content is `some content`.
        full: bool,
    },
    /// Document tag.
    ///
    /// These are tags with exclamation marks
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    Document {
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
    Tag {
        /// Opening tag
        ///
        /// Contains the name of the tag and its attributes.
        tag: Tag,
        /// Type of the tag
        ///
        /// The type is the information on the closing style: self-closing (`<div/>`), opened (`<div>`) or closed (`<div></div>`).
        full: TagType,
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
    /// In `a<strong>b`, the node is a vector, with [`Html::Text`] `a`, [`Html::Tag`] `strong` [`Html::Text`] `b`.
    Vec(Vec<Html>),
}

impl Html {
    /// Pushes a block comment into the html tree
    pub(crate) fn close_comment(&mut self) -> bool {
        match self {
            Self::Comment { full, .. } => {
                if *full {
                    false
                } else {
                    *full = true;
                    true
                }
            }
            Self::Text(_) | Self::Empty | Self::Document { .. } => false,
            Self::Tag { full, child, .. } => full.is_open() && child.close_comment(),
            Self::Vec(vec) => vec.last_mut().map_or_else(|| false, Self::close_comment),
        }
    }

    /// Method to find to close that last opened tag.
    ///
    /// This method finds the opened tag the closest to the leaves.
    pub(crate) fn close_tag(&mut self, name: &str) -> Result<(), String> {
        match self.close_tag_aux(name) {
            TagClosingStatus::Success => Ok(()),
            TagClosingStatus::Full => Err(format!(
                "Invalid closing tag: Found closing tag for '{name}' but all tags are already closed."
            )),
            TagClosingStatus::WrongName(expected) => Err(format!(
                "Invalid closing tag: Found closing tag for '{name}' but '{expected}' is still open."
            )),
        }
    }

    /// Wrapper for [`Self::close_tag`].
    pub(crate) fn close_tag_aux(&mut self, name: &str) -> TagClosingStatus {
        if let Self::Tag {
            tag,
            full: full @ TagType::Opened,
            child,
        } = self
        {
            let status = child.close_tag_aux(name);
            if matches!(status, TagClosingStatus::Full) {
                if tag.name == name {
                    *full = TagType::Closed;
                    TagClosingStatus::Success
                } else {
                    TagClosingStatus::WrongName(take(&mut tag.name))
                }
            } else {
                status
            }
        } else if let Self::Vec(vec) = self {
            vec.last_mut()
                .map_or(TagClosingStatus::Full, |child| child.close_tag_aux(name))
        } else {
            TagClosingStatus::Full
        }
    }

    /// Boxes an empty tree.
    pub(crate) fn empty_box() -> Box<Self> {
        Box::new(Self::default())
    }

    /// Creates a tree for a character.
    pub(crate) fn from_char(ch: char) -> Self {
        Self::Text(ch.to_string())
    }

    /// Checks if the writer is currently in a comment
    pub(crate) fn is_comment(&self) -> bool {
        match self {
            Self::Comment { full, .. } => !*full,
            Self::Empty | Self::Text(_) | Self::Document { .. } => false,
            Self::Tag { full, child, .. } => full.is_open() && child.is_comment(),
            Self::Vec(vec) => vec.last().is_some_and(Self::is_comment),
        }
    }

    /// Checks if an html tree is empty.
    ///
    /// This is equivalent to check if tree is [`Html::Empty`] as all the others are initialised with at least one character.
    const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Checks if an html tree is pushable.
    ///
    /// This is to check if a new node needs to be created for the next data.
    ///
    /// This method is different if the input is a char or not.
    pub(crate) const fn is_pushable(&self, is_char: bool) -> bool {
        match self {
            Self::Empty | Self::Vec(_) => true,
            Self::Tag { full, .. } => full.is_open(),
            Self::Document { .. } => false,
            Self::Text(_) => is_char,
            Self::Comment { full, .. } => !*full,
        }
    }

    /// Pushes one character into an html tree.
    pub(crate) fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => *self = Self::from_char(ch),
            Self::Tag {
                child,
                full: TagType::Opened,
                ..
            } => child.push_char(ch),
            Self::Document { .. }
            | Self::Tag {
                full: TagType::Closed | TagType::SelfClosing,
                ..
            } => *self = Self::Vec(vec![take(self), Self::from_char(ch)]),
            Self::Text(text) => text.push(ch),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(true) {
                        return last.push_char(ch);
                    }
                }
                vec.push(Self::from_char(ch));
            }
            Self::Comment { content, full } => {
                if *full {
                    // This means the comment is at the root
                    *self = Self::Vec(vec![take(self), Self::from_char(ch)]);
                } else {
                    content.push(ch);
                }
            }
        }
    }

    /// Pushes a block comment into the html tree
    pub(crate) fn push_comment(&mut self) {
        self.push_node(Self::Comment {
            content: String::new(),
            full: false,
        });
    }

    /// Pushes an html tree into another one.
    ///
    /// This is useful to add comments or push tags for instance.
    pub(crate) fn push_node(&mut self, node: Self) {
        match self {
            Self::Empty => *self = node,
            Self::Tag {
                child,
                full: TagType::Opened,
                ..
            } => child.push_node(node),
            Self::Text(_)
            | Self::Document { .. }
            | Self::Tag {
                full: TagType::Closed | TagType::SelfClosing,
                ..
            } => *self = Self::Vec(vec![take(self), node]),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(false) {
                        return last.push_node(node);
                    }
                }
                vec.push(node);
            }
            Self::Comment { .. } => {
                safe_unreachable!("Pushed parsed not into an unclosed comment.")
            }
        }
    }

    /// Pushes a tag into an html tree.
    pub(crate) fn push_tag(&mut self, tag: Tag, inline: bool) {
        self.push_node(Self::Tag {
            tag,
            full: if inline {
                TagType::SelfClosing
            } else {
                TagType::Opened
            },
            child: Self::empty_box(),
        });
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Html {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f)?,
            Self::Tag { tag, full, child } => match full {
                TagType::Closed => {
                    write!(f, "<{tag}>{child}</{}>", tag.name)
                }
                TagType::Opened => {
                    write!(f, "<{tag}>{child}")
                }
                TagType::SelfClosing => {
                    debug_assert!(child.is_empty(), "child can't be pushed if inline");
                    write!(f, "<{tag} />")
                }
            }?,
            Self::Document { name, attr } => match (name, attr) {
                (name_str, None) if name_str.is_empty() => write!(f, "<!>"),
                (name_str, None) => write!(f, "<!{name_str} >"),
                (name_str, Some(attr_str)) => write!(f, "<!{name_str} {attr_str}>"),
            }?,
            Self::Text(text) => text.fmt(f)?,
            Self::Vec(vec) => {
                for html in vec {
                    html.fmt(f)?;
                }
            }
            Self::Comment { content, full } => {
                f.write_str("<!--")?;
                f.write_str(content)?;
                if *full {
                    f.write_str("-->")?;
                }
            }
        }
        Ok(())
    }
}
