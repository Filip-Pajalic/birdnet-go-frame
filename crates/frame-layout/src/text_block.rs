//! One run of text placed on the page.

use alloc::string::String;

use crate::align::Align;
use crate::text_style::TextStyle;

/// A single line of text and where it goes.
#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    /// The text, one line.
    pub text: String,
    /// Horizontal anchor in pixels; see [`TextBlock::align`].
    pub x: i32,
    /// Baseline in pixels from the top of the page.
    pub baseline_y: i32,
    /// Font size in pixels.
    pub size_px: f32,
    /// Widest the text may be, in pixels; the renderer shrinks the font to fit. `None` for no
    /// limit.
    pub max_width: Option<i32>,
    /// Which edge `x` marks.
    pub align: Align,
    /// Role, which decides the ink.
    pub style: TextStyle,
}
