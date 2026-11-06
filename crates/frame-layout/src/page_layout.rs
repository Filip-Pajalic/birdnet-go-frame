//! A whole page, ready to draw.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::image_block::ImageBlock;
use crate::text_block::TextBlock;

/// Everything to draw on one page, as the viewer sees it. Pictures are drawn first, then text.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLayout {
    /// Page size as the viewer sees it.
    pub resolution: Resolution,
    /// Pictures to draw, in order.
    pub images: Vec<ImageBlock>,
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
            text: Vec::new(),
        }
    }
}
