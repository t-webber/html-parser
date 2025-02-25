//! Implements the ownership model to handle both own and referenced data.

use crate::prelude::Html;

/// Contains either an owned or borrowed value
///
/// Useful to implement methods on both references and owned data.
pub enum Ownership<'borrow, T> {
    Borrowed(&'borrow T),
    Owned(T),
}

impl<T: ToOwned<Owned = T>> Ownership<'_, T> {
    /// Returns owned data, either by returning the value or cloning
    #[inline]
    pub fn into_owned(self) -> T {
        match self {
            Self::Borrowed(borrowed) => borrowed.to_owned(),
            Self::Owned(owned) => owned,
        }
    }
}

impl<T> From<T> for Ownership<'_, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'borrow, T> From<&'borrow T> for Ownership<'borrow, T> {
    fn from(value: &'borrow T) -> Self {
        Self::Borrowed(value)
    }
}

impl<T> AsRef<T> for Ownership<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(borrowed) => *borrowed,
            Self::Owned(owned) => owned,
        }
    }
}

impl<T> Ownership<'_, Box<[T]>> {
    pub fn into_iter_map_collect<U, F: Fn(Ownership<'_, T>) -> U>(self, map: F) -> Box<[U]> {
        match self {
            Self::Borrowed(borrowed) => borrowed.into_iter().map(|elt| map(elt.into())).collect(),
            Self::Owned(owned) => owned.into_iter().map(|elt| map(elt.into())).collect(),
        }
    }
    pub fn into_iter_filter_map_collect<
        U,
        V: FromIterator<U>,
        F: Fn(Ownership<'_, T>) -> Option<U>,
    >(
        self,
        map: F,
    ) -> V {
        match self {
            Self::Borrowed(borrowed) => borrowed
                .into_iter()
                .filter_map(|elt| map(elt.into()))
                .collect(),
            Self::Owned(owned) => owned
                .into_iter()
                .filter_map(|elt| map(elt.into()))
                .collect(),
        }
    }
}
