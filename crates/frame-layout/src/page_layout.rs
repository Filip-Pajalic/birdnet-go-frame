//! A whole page, ready to draw.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::image_block::ImageBlock;
use crate::shape_block::ShapeBlock;
use crate::text_block::TextBlock;

/// Everything to draw on one page, as the viewer sees it: pictures first, then shapes, then
/// text.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLayout {
    /// Page size as the viewer sees it.
    pub resolution: Resolution,
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
            images: Vec::new(),
            shapes: Vec::new(),
            text: Vec::new(),
        }
    }
}
