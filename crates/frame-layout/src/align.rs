//! Horizontal alignment of a text block.

/// Which edge of a text block its `x` coordinate marks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// `x` is the left edge.
    #[default]
    Left,
    /// `x` is the right edge.
    Right,
    /// `x` is the centre.
    Centre,
}
