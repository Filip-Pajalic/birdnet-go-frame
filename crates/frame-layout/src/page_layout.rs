//! A whole page, ready to draw.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::image_block::ImageBlock;
use crate::pen_mark::PenMark;
use crate::shape_block::ShapeBlock;
use crate::text_block::TextBlock;

/// Everything to draw on one page, as the viewer sees it. Ornaments are drawn first, then
/// pictures, then shapes, then text.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLayout {
    /// Page size as the viewer sees it.
    pub resolution: Resolution,
    /// Ornaments' pen marks, such as a [`crate::SeasonBorder`], in order.
    pub ornament: Vec<PenMark>,
    /// Pictures to draw, in order.
    pub images: Vec<ImageBlock>,
    /// Filled rectangles to draw, in order.
    pub shapes: Vec<ShapeBlock>,
    /// Text to draw, in order.
    pub text: Vec<TextBlock>,
}

impl PageLayout {
    /// Creates an empty page.
    #[must_use]
    pub fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            ornament: Vec::new(),
            images: Vec::new(),
            shapes: Vec::new(),
            text: Vec::new(),
        }
    }
}
