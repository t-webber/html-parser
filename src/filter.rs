//! Module to define structs to filter

use std::cmp::Ordering;
use std::collections::HashSet;

use crate::errors::safe_unreachable;
use crate::types::html::Html;
use crate::types::tag::{Attribute, PrefixName, Tag};

/// Macro to setup a filter
macro_rules! filter_setter {
    ($($name:ident)*) => {
       $(
            #[doc = concat!("Activates the ", stringify!($name), "s in the filter")]
            #[inline]
            #[must_use]
            /// Activates the specified field for filtering.
            pub const fn $name(mut self, $name: bool) -> Self {
                self.types.$name = $name;
                self
            }
        )*
    };
}

/// Data structure to defines the filters to select the wanted elements of the
/// Html tree
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Filter {
    /// Attributes of the wanted tags
    attrs: Option<HashSet<Attribute>>,
    /// Depth in which to embed the required nodes
    ///
    /// # Examples
    ///
    /// If the html is `<nav><ul><li>Click on the <a
    /// href="#">link</a><li></ul></nav>` and we search with the filter
    ///
    /// ```
    /// Filter::default().depth(1).tag_name("a");
    /// ```
    ///
    /// the expected output is `<li>Click on the <a href="#">link</a><li>`.
    ///
    /// - If the depth were 0, the output would have been only the `a` tag.
    /// - If the depth were 2, the output would have been the whole the `ul`
    ///   tag.
    depth: usize,
    /// Html tags
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    tags: Option<HashSet<String>>,
    /// Filter by type of html node
    types: HtmlFilterType,
}

#[expect(clippy::arbitrary_source_item_ordering, reason = "macro used")]
impl Filter {
    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn attribute_name<N: Into<String>>(mut self, name: N) -> Self {
        let attr = Attribute::NameNoValue(PrefixName::from(name.into()));
        if let Some(attrs) = &mut self.attrs {
            attrs.insert(attr);
        } else {
            let mut hash_set = HashSet::new();
            hash_set.insert(attr);
            self.attrs = Some(hash_set);
        }
        self
    }

    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn attribute_value<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        let attr = Attribute::NameValue {
            name: PrefixName::from(name.into()),
            value: value.into(),
            double_quote: true,
        };
        if let Some(attrs) = &mut self.attrs {
            attrs.insert(attr);
        } else {
            let mut hash_set = HashSet::new();
            hash_set.insert(attr);
            self.attrs = Some(hash_set);
        }
        self
    }

    //TODO: only works if either one if empty
    /// Method to check all the attributes are present.
    fn allowed_tag(&self, tag: &Tag) -> bool {
        self.tags
            .as_ref()
            .is_some_and(|names| names.contains(&tag.name))
            || self
                .attrs
                .as_ref()
                .is_some_and(|wanted| wanted.iter().all(|attr| tag.attrs.contains(attr)))
    }
    #[inline]
    #[must_use]
    /// Activates everything, except if tag names or attributes were given.
    pub const fn all(mut self) -> Self {
        self.types.comment = true;
        self.types.document = true;
        self
    }

    filter_setter!(comment document);

    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn tag_name<N: Into<String>>(mut self, name: N) -> Self {
        if let Some(names) = &mut self.tags {
            names.insert(name.into());
        } else {
            let mut names = HashSet::new();
            names.insert(name.into());
            self.tags = Some(names);
        }
        self
    }
}

/// State to follow if the wanted nodes where found at what depth
#[derive(Default, Debug)]
enum DepthSuccess {
    /// Not wanted node, doesn't respect the filters
    #[default]
    None,
    Found(usize),
    Success,
}

impl DepthSuccess {
    fn incr(&mut self) {
        if let Self::Found(depth) = self {
            *depth += 1;
        }
    }

    fn current(&self) -> Option<usize> {
        if let Self::Found(depth) = self {
            Some(*depth)
        } else {
            None
        }
    }
}

/// Status of the filtering on recursion calls
#[derive(Default, Debug)]
struct FilterSuccess {
    /// Indicates if the filter found a wanted node
    ///
    /// Is
    /// - `None` if no wanted node was found
    /// - `Some(depth)` if a wanted node was found at depth `depth`. If there
    ///   are embedded nodes that satisfy the filter, `depth` is the smallest
    ///   possible.
    depth: DepthSuccess,
    /// Result of the filtering
    html: Html,
}

impl FilterSuccess {
    /// Creates a [`FilterSuccess`] from an [`Html`]
    fn found(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::Found(0), html })
    }

    /// Increment the depth, if applicable
    fn incr(mut self) -> Option<Self> {
        self.depth.incr();
        Some(self)
    }

    //
    fn is_found(&self) -> bool {
        !matches!(self.depth, DepthSuccess::None)
    }
}

impl Html {
    /// Filters html based on a defined filter.
    #[inline]
    #[must_use]
    pub fn filter(self, filter: &Filter) -> Self {
        self.filter_aux(filter).html
    }

    /// Wrapper for [`Self::filter`]
    ///
    /// Refer to [`Self::filter`] for documentation.
    ///
    /// This methods takes an additional `clean` boolean to indicate when a tag
    /// returns the child. In that case, the texts must disappear if present at
    /// root.
    ///
    /// This methods returns a wrapper of the final html in a [`FilterSuccess`]
    /// to follow the current depth of the last found node. See
    /// [`FilterSuccess`] for more information.
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "incr depth when smaller than filter_depth"
    )]
    fn filter_aux(self, filter: &Filter) -> FilterSuccess {
        let input: String = format!("{self:?}").chars().take(100).collect();
        let output = match self {
            Self::Comment { .. } if !filter.types.comment => None,
            Self::Document { .. } if !filter.types.document => None,
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => None,

            Self::Tag { ref tag, .. } if filter.allowed_tag(tag) => FilterSuccess::found(self),
            Self::Tag { child, .. } if filter.depth == 0 => child.filter_aux(filter).incr(),
            Self::Tag { child, tag, full } => {
                let rec = child.filter_aux(filter);
                match rec.depth {
                    DepthSuccess::None => None,
                    DepthSuccess::Success => Some(rec),
                    DepthSuccess::Found(depth) => match depth.cmp(&filter.depth) {
                        Ordering::Less => Some(FilterSuccess {
                            depth: DepthSuccess::Found(depth + 1),
                            html: Self::Tag { tag, full, child: Box::new(rec.html) },
                        }),
                        Ordering::Equal => match rec.html {
                            Self::Tag { .. } =>    Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
                            Self::Vec(_) => Some(FilterSuccess {
                                depth: DepthSuccess::Found(depth + 1),
                                html: Self::Tag { tag, full, child: Box::new(rec.html) },
                            }),
                                Self::Empty
                            | Self::Text(_)
                            | Self::Comment { .. }
                            | Self::Document { .. } => safe_unreachable("filter depth is not 0"),
                        },
                        Ordering::Greater =>
                            Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
                    },
                }
            }

            Self::Vec(vec) => {
                let filtered_vec = vec
                    .into_iter()
                    .map(|child| child.filter_aux(filter))
                    .filter(|child| !child.html.is_empty())
                    .collect::<Vec<_>>();
                filtered_vec
                    .iter()
                    .filter_map(|child| child.depth.current())
                    .min()
                    .map(|depth| FilterSuccess {
                        depth: if depth < filter.depth {
                            DepthSuccess::Found(depth + 1)
                        } else {
                            DepthSuccess::Success
                        },
                        html: Self::Vec({let vec = if depth < filter.depth {
                            filtered_vec.into_iter().map(|child| child.html).collect()
                        } else {
                            filtered_vec
                                .into_iter()
                                .filter(FilterSuccess::is_found)
                                .map(|child| child.html)
                                .collect()
                        }; if vec.len()<= 1 {
                            vec.pop().unwrap_or_default()
                        } else {
                            vec
                        }}),
                    })
            }

            Self::Text(_) => Some(FilterSuccess{html:self, depth: DepthSuccess::None }),
            Self::Empty => None, 
            Self::Comment { .. } | Self::Document { .. }  =>
                FilterSuccess::found(self),
        }
        .unwrap_or_default();
        let oustr: String = format!("{output:?}").chars().take(100).collect();
        println!(
            "----------------------------------------\n{input}\n\t=>\n{oustr}\n----------------------------------------"
        );
        output
    }

    /// Filters html based on a defined filter.
    #[inline]
    #[must_use]
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    pub fn find(self, filter: &Filter) -> Option<Self> {
        match self {
            Self::Comment { .. } if !filter.types.comment => None,
            Self::Document { .. } if !filter.types.document => None,
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => None,

            Self::Tag { ref tag, .. } if filter.allowed_tag(tag) => Some(self),
            Self::Tag { child, .. } => child.find(filter),
            Self::Vec(vec) => {
                for child in vec {
                    if let Some(found) = child.find(filter) {
                        return Some(found);
                    }
                }
                None
            }
            Self::Comment { .. } | Self::Document { .. } | Self::Empty | Self::Text(_) => None,
        }
    }
}

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[derive(Default, Debug)]
struct HtmlFilterType {
    /// Html comment
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    comment: bool,
    /// Html document tags
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    document: bool,
}
