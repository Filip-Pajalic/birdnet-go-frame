//! A whole page, ready to draw.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::text_block::TextBlock;

/// Everything to draw on one page, as the viewer sees it.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLayout {
    /// Page size as the viewer sees it.
    pub resolution: Resolution,
    /// Text to draw, in order.
    pub text: Vec<TextBlock>,
}
