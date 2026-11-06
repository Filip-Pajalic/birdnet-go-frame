//! The frame's three buttons.

/// A button on the frame, as the EE02 labels them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    /// Draws the page again now, even if it has not changed. Held at power-up, it starts the
    /// setup hotspot instead.
    Refresh,
    /// Steps back to the previous [`PageView`](crate::PageView).
    Previous,
    /// Steps on to the next [`PageView`](crate::PageView).
    Next,
}
