//! Module to filter an HTML tree to keep or remove specific nodes, with a set
//! of rules.
//!
//! You can either filter your HTML with [`Html::filter`] or find a specific
//! node with [`Html::find`].
//!
//! For more information on how to define the filtering rules, please refer to
//! [`Filter`].

mod element;
pub mod types;

use core::cmp::Ordering;

use types::{Filter, HtmlFilterType};

use crate::safe_expect;
use crate::types::html::Html;

/// State to follow if the wanted nodes where found at what depth
///
/// # Note
///
/// We implement the discriminant and specify the representation size in order
/// to derive [`Ord`] trait.
#[repr(u8)]
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum DepthSuccess {
    /// Wanted node wanting more depth
    Found(usize) = 1,
    /// Not wanted node, doesn't respect the filters
    #[default]
    None = 2,
    /// Wanted node with already the wanted depth
    Success = 0,
}

impl DepthSuccess {
    /// Increment the depth, if applicable
    #[inline]
    #[coverage(off)]
    fn incr(mut self) -> Self {
        if let Self::Found(depth) = &mut self {
            *depth = safe_expect!(depth.checked_add(1), "Smaller than required depth");
        }
        self
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
    /// Increment the depth, if applicable
    #[inline]
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    fn incr(mut self) -> Option<Self> {
        self.depth = self.depth.incr();
        Some(self)
    }

    /// Creates a [`FilterSuccess`] from an [`Html`]
    ///
    /// This is the method to use when the node is considered `found`, i.e.,
    /// when it was the node the user was looking for.
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    const fn make_found(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::Found(0), html })
    }

    /// Creates a [`FilterSuccess`] from an [`Html`]
    ///
    /// This is the method to use when the node isn't interesting alone, it can
    /// be if it is in the right scope though.
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    const fn make_none(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::None, html })
    }
}

impl Html {
    /// Method to check if a wanted node is visible
    ///
    /// This methods stop checking after a maximum depth, as the current node
    /// will be discarded if it is deeper in the tree.
    // TODO: users can implement this an be disappointed
    fn check_depth(&self, max_depth: usize, filter: &Filter) -> Option<usize> {
        match self {
            Self::Empty | Self::Text(_) | Self::Comment { .. } | Self::Document { .. } => None,
            Self::Tag { tag, .. } if filter.tag_allowed(tag) => Some(0),
            Self::Tag { .. } | Self::Vec(_) if max_depth == 0 => None,
            Self::Tag { child, .. } => child
                .check_depth(
                    #[expect(clippy::arithmetic_side_effects, reason = "non-0")]
                    {
                        max_depth - 1
                    },
                    filter,
                )
                .map(
                    #[expect(clippy::arithmetic_side_effects, reason = "< initial max_depth")]
                    |depth| depth + 1,
                ),
            Self::Vec(vec) => vec
                .iter()
                .try_fold(Some(usize::MAX), |acc, child| {
                    if acc == Some(0) {
                        Err(())
                    } else {
                        Ok(child.check_depth(max_depth, filter))
                    }
                })
                .unwrap_or(Some(0)),
        }
    }

    /// Filters html based on a defined filter.
    ///
    /// See [`Filter`] to learn how to create filters.
    ///
    /// Filters allow you to select the portions of the html code you want to
    /// keep or remove.
    ///
    /// # Returns
    ///
    /// The html tree obtains by keeping only the nodes that fulfills the
    /// filter.
    #[inline]
    #[must_use]
    pub fn filter(self, filter: &Filter) -> Self {
        self.filter_aux(filter, false).html
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
    fn filter_aux(self, filter: &Filter, found: bool) -> FilterSuccess {
        // let input = format!("{self:?}").chars().take(150).collect::<String>();
        let output = match self {
            Self::Comment { .. } if found || !filter.comment_allowed() => None,
            Self::Document { .. } if found || !filter.document_allowed() => None,
            Self::Text(txt)
                if found || !filter.text_allowed() || txt.chars().all(char::is_whitespace) =>
                None,

            Self::Tag { ref tag, .. } if filter.tag_allowed(tag) =>
                FilterSuccess::make_found(self.filter_light(filter.as_types())),
            Self::Tag { child, .. } if filter.as_depth() == 0 =>
                child.filter_aux(filter, found).incr(),
            Self::Tag { child, tag, full } => {
                let rec = child.filter_aux(filter, found);
                match rec.depth {
                    DepthSuccess::None => None,
                    DepthSuccess::Success => Some(rec),
                    DepthSuccess::Found(depth) => match depth.cmp(&filter.as_depth()) {
                        Ordering::Less => Some(FilterSuccess {
                            depth: DepthSuccess::Found(depth + 1),
                            html: Self::Tag { tag, full, child: Box::new(rec.html) },
                        }),
                        Ordering::Equal | Ordering::Greater =>
                            Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
                    },
                }
            }

            Self::Vec(vec) => {
                match vec
                    .iter()
                    .filter_map(|child| child.check_depth(filter.as_depth() + 1, filter))
                    .collect::<Vec<_>>()
                    .iter()
                    .min()
                {
                    Some(depth) if *depth < filter.as_depth() => Some(FilterSuccess {
                        depth: DepthSuccess::Found(*depth),
                        html: Self::Vec(
                            vec.into_iter()
                                .map(|child| child.filter_light(filter.as_types()))
                                .collect(),
                        ),
                    }),
                    Some(_) => Some(FilterSuccess {
                        depth: DepthSuccess::Success,
                        html: Self::Vec(
                            vec.into_iter()
                                .map(|child| child.filter_aux(filter, true))
                                .filter(|child| !child.html.is_empty())
                                .map(|child| child.html)
                                .collect::<Vec<_>>(),
                        ),
                    }),
                    None => {
                        let mut filtered = vec
                            .into_iter()
                            .map(|child| child.filter_aux(filter, false))
                            .filter(|node| !node.html.is_empty())
                            .collect::<Vec<_>>();
                        if filtered.len() <= 1 {
                            filtered.pop()
                        } else {
                            filtered.iter().map(|child| child.depth).min().map(|depth| {
                                FilterSuccess {
                                    depth,
                                    html: Self::Vec(
                                        filtered.into_iter().map(|child| child.html).collect(),
                                    ),
                                }
                            })
                        }
                    }
                }
            }

            Self::Text(_) | Self::Empty => None,
            Self::Comment { .. } | Self::Document { .. } => FilterSuccess::make_none(self),
        }
        .unwrap_or_default();
        //         println!(
        //             "
        // ------------------------------------------------------
        // {input}
        // =>
        // {:?}\n{}
        // ------------------------------------------------------
        //         ",
        //             output.depth, output.html
        //         );
        output
    }

    /// Light filter without complicated logic, just filtering on types.
    ///
    /// This method does take into account the [`Filter::tag_name`],
    ///   [`Filter::attribute_name`] and [`Filter::attribute_value`] methods,
    /// only the types of [`HtmlFilterType`].
    #[coverage(off)]
    fn filter_light(self, filter: &HtmlFilterType) -> Self {
        match self {
            Self::Text(_) if filter.text => self,
            Self::Comment { .. } if filter.comment => self,
            Self::Document { .. } if filter.document => self,
            Self::Tag { tag, full, child } =>
                Self::Tag { tag, full, child: Box::new(child.filter_light(filter)) },
            Self::Vec(vec) => Self::Vec(
                vec.into_iter()
                    .map(|child| child.filter_light(filter))
                    .collect(),
            ),
            Self::Empty | Self::Text(_) | Self::Comment { .. } | Self::Document { .. } =>
                Self::Empty,
        }
    }

    /// Finds an html node based on a defined filter.
    ///
    /// See [`Filter`] to know how to define a filter.
    ///
    /// Filters allow you to select the portions of the html code you want to
    /// keep or remove.
    ///
    /// # Returns
    ///
    /// The first node that fulfills the filter.
    #[inline]
    #[must_use]
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    pub fn find(self, filter: &Filter) -> Option<Self> {
        match self {
            Self::Comment { .. } if !filter.comment_allowed() => None,
            Self::Document { .. } if !filter.document_allowed() => None,
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => None,

            Self::Tag { ref tag, .. } if filter.tag_allowed(tag) => Some(self),
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
