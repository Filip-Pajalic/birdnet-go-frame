//! The heading and footer every page shares.

use alloc::string::String;

use crate::align::Align;
use crate::page_layout::PageLayout;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// Typical width of a letter in the bundled italic, as a fraction of the font size.
pub(crate) const AVERAGE_LETTER_WIDTH: f32 = 0.42;

/// The space between the heading and the footer, and the sizes chosen for the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Body {
    pub(crate) margin: i32,
    pub(crate) width: i32,
    pub(crate) top: i32,
    pub(crate) bottom: i32,
    pub(crate) heading_size: f32,
    pub(crate) footer_size: f32,
}

/// Adds the heading and footer to `page` and returns the space left between them. An empty
/// heading or footer is left out and its space given to the body.
pub(crate) fn frame(page: &mut PageLayout, heading: &str, footer: &str) -> Body {
    let width: i32 = i32::try_from(page.resolution.width).unwrap_or(i32::MAX);
    let height: i32 = i32::try_from(page.resolution.height).unwrap_or(i32::MAX);
    let margin: i32 = width.min(height) / 16;
    let heading_size: f32 = (height as f32 / 12.0).clamp(32.0, 120.0);
    let footer_size: f32 = (heading_size * 0.3).max(16.0);
    if !heading.is_empty() {
        page.text.push(TextBlock {
            text: String::from(heading),
            x: width / 2,
            baseline_y: margin + heading_size as i32,
            size_px: heading_size,
            max_width: Some(width - 2 * margin),
            align: Align::Centre,
            style: TextStyle::Heading,
        });
    }
    if !footer.is_empty() {
        page.text.push(TextBlock {
            text: String::from(footer),
            x: width / 2,
            baseline_y: height - margin,
            size_px: footer_size,
            max_width: Some(width - 2 * margin),
            align: Align::Centre,
            style: TextStyle::Footer,
        });
    }
    Body {
        margin,
        width,
        top: if heading.is_empty() {
            margin
        } else {
            margin + (heading_size * 1.8) as i32
        },
        bottom: if footer.is_empty() {
            height - margin
        } else {
            height - margin - (footer_size * 1.5) as i32
        },
        heading_size,
        footer_size,
    }
}

/// Adds `message`, the line shown when nothing has been heard.
pub(crate) fn nothing_heard(page: &mut PageLayout, body: &Body, message: &str) {
    page.text.push(TextBlock {
        text: String::from(message),
        x: body.width / 2,
        baseline_y: (body.top + body.bottom) / 2,
        size_px: body.heading_size * 0.6,
        max_width: Some(body.width - 2 * body.margin),
        align: Align::Centre,
        style: TextStyle::Detail,
    });
}
