//! What showing a page came to.

/// How showing a page went: the refresh, and putting the panel to sleep afterwards, which is
/// tried whatever the refresh did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct Refresh<E> {
    /// How many milliseconds the refresh took, or why the page is not on the glass.
    pub shown: Result<u32, E>,
    /// Whether the panel went to sleep. The picture stays on the glass either way.
    pub slept: Result<(), E>,
}
