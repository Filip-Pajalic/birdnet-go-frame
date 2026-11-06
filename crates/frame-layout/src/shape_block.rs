//! A filled rectangle placed on the page, for charts and rules.

use frame_core::Ink;

/// A rectangle filled with one ink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeBlock {
    /// Left edge in pixels.
    pub x: i32,
    /// Top edge in pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// The ink it is filled with.
    pub ink: Ink,
}
