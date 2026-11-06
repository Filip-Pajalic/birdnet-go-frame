//! A stroke of the engraver's pen, for ornaments.

use frame_core::Ink;

use crate::point::Point;

/// One hard-edged mark in one ink: a line with round ends, or a filled dot. Ornaments such as
/// the seasonal border are made of many, drawn in order under the pictures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenMark {
    /// A straight line with round ends.
    Line {
        /// Where it starts.
        from: Point,
        /// Where it ends.
        to: Point,
        /// Its width in pixels.
        width: f32,
        /// Its ink.
        ink: Ink,
    },
    /// A filled circle.
    Dot {
        /// Its centre.
        centre: Point,
        /// Its radius in pixels.
        radius: f32,
        /// Its ink.
        ink: Ink,
    },
}
