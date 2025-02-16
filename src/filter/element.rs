//! Keeps track of rules applied on attributes or tags. They can
//! either be blacklisted or whitelisted by the user. This module handles the
//! logic for the combination of these rules.

use core::hash::Hash;
use std::collections::HashMap;

/// Stores the status of an element, i.e., whether it ought to be kept or
/// removed.
///
/// This contains only the explicit rules given by the user at the definition of
/// [`super::Filter`].
///
/// It contains a `whitelist` and a `blacklist` to keep track of the filtering
/// parameters.
#[derive(Debug)]
pub struct ElementFilter<T> {
    /// Contains the elements and their status
    ///
    /// The hashmap maps a name to a target, and a bool. The boolean is `true`
    /// if the item is whitelisted, and `false` if the item is blacklisted.
    items: HashMap<String, (T, bool)>,
    /// Indicates if a whitelisted element was pushed into the [`HashMap`].
    whitelist_empty: bool,
}

impl<T: Eq + Hash> ElementFilter<T> {
    /// Check the status of an element
    pub fn check<F: Fn(&T) -> bool>(
        &self,
        name: &String,
        test_value: &F,
        must_contain: bool,
    ) -> ElementState {
        self.items.get(name).map_or_else(
            || {
                if must_contain && !self.whitelist_empty {
                    ElementState::BlackListed
                } else {
                    ElementState::NotSpecified
                }
            },
            |(target, keep)| match (test_value(target), keep) {
                (true, true) => ElementState::WhiteListed,
                (true, false) | (false, true) => ElementState::BlackListed,
                (false, false) => ElementState::NotSpecified,
            },
        )
    }

    /// Pushes an element as whitelisted or blacklisted
    pub fn push(&mut self, name: String, value: T, keep: bool) {
        self.items.insert(name, (value, keep));
        if keep {
            self.whitelist_empty = false;
        }
    }
}

impl<T> Default for ElementFilter<T> {
    fn default() -> Self {
        Self { items: HashMap::new(), whitelist_empty: true }
    }
}

/// Status of an element
///
/// An element can be whitelisted or blacklisted by the user. This state
/// contains both information.
#[derive(Debug)]
pub enum ElementState {
    /// Element ought to be removed
    BlackListed,
    /// No rules applied for this element
    NotSpecified,
    /// Element ought to be kept
    WhiteListed,
}

impl ElementState {
    /// Computes the output status for multiple checks
    ///
    /// This is used to perform multiple successive tests.
    pub const fn and(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::BlackListed, _) | (_, Self::BlackListed) => Self::BlackListed,
            (Self::NotSpecified, Self::NotSpecified) => Self::NotSpecified,
            // in this arm, at least one is WhiteListed, because the other case is above.
            (Self::WhiteListed | Self::NotSpecified, Self::WhiteListed | Self::NotSpecified) =>
                Self::WhiteListed,
        }
    }

    /// Checks if an element was explicitly authorised, i.e., is whitelisted
    pub const fn is_explicitly_authorised(&self) -> bool {
        matches!(self, Self::WhiteListed)
    }
}
