//! The heading, footer and border every page shares.

use alloc::string::String;
use alloc::vec::Vec;

use crate::align::Align;
use crate::corner_sprays::{CornerSprays, PlacedSpray, Spray};
use crate::page_layout::PageLayout;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// Typical width of a letter in the bundled italic, as a fraction of the font size.
pub(crate) const AVERAGE_LETTER_WIDTH: f32 = 0.42;

/// The most of the page's short side a corner spray may take beside a heading or footer.
const SPRAY_SHARE: f32 = 0.24;
/// A spray's side on a page whose body is packed round it (the collage), as a share of the
/// short side.
const PACKED_ROUND_SHARE: f32 = 0.3;
/// Space between a spray and the page's edges, as a share of the short side.
const SPRAY_INSET: f32 = 0.012;

/// The space between the heading and the footer, the sizes chosen for the page, and the
/// border's sprays, which the body must keep clear of.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Body {
    pub(crate) margin: i32,
    pub(crate) width: i32,
    pub(crate) top: i32,
    pub(crate) bottom: i32,
    pub(crate) heading_size: f32,
    pub(crate) footer_size: f32,
    /// The border's sprays as placed.
    pub(crate) sprays: Vec<PlacedSpray>,
}

/// The words at the top and foot of a page, and its border. Any word may be empty, and is then
/// left out.
pub(crate) struct Furniture<'a> {
    pub(crate) heading: &'a str,
    /// A line about the page's main bird, over the footer.
    pub(crate) fact: &'a str,
    pub(crate) footer: &'a str,
    pub(crate) sprays: &'a CornerSprays,
    /// Whether the body is packed round the sprays, so they need not keep to the heading's
    /// and footer's corners.
    pub(crate) packed_round: bool,
    /// The heading's and the fact's widths at a size of one pixel, when the page knows them:
    /// a line then sits in the middle of the page, moved aside only as far as its corner's
    /// spray needs. 0 for a line centred in the span the spray leaves.
    pub(crate) widths_em: (f32, f32),
}

/// Returns where to centre a line `text_width` wide that must stay within `span` on a page
/// `page_width` wide: the middle of the page when it fits there, else as near it as the span
/// allows. A width of 0 (unknown) or one wider than the span centres it in the span.
fn centre_in(span: (i32, i32), page_width: i32, text_width: f32) -> i32 {
    let room: i32 = span.1 - span.0;
    let half: i32 = libm::ceilf(text_width / 2.0) as i32;
    if half <= 0 || 2 * half > room {
        return (span.0 + span.1) / 2;
    }
    (page_width / 2).clamp(span.0 + half, span.1 - half)
}

/// Adds the heading, fact, footer and border to `page` and returns the space left between
/// them. An empty word is left out and its space given to the body. The top-left spray fills
/// the corner beside the heading and the bottom-right one the corner beside the footer, which
/// are centred in the room the sprays leave; the body's own text never reaches those corners. A body
/// `packed_round` the sprays, the collage's, lets them grow larger.
pub(crate) fn frame(page: &mut PageLayout, words: &Furniture<'_>) -> Body {
    let Furniture {
        heading,
        fact,
        footer,
        sprays,
        packed_round,
        widths_em: (heading_em, fact_em),
    } = *words;
    let width: i32 = i32::try_from(page.resolution.width).unwrap_or(i32::MAX);
    let height: i32 = i32::try_from(page.resolution.height).unwrap_or(i32::MAX);
    let short: f32 = width.min(height) as f32;
    let margin: i32 = width.min(height) / 16;
    let heading_size: f32 = (height as f32 / 12.0).clamp(32.0, 120.0);
    let footer_size: f32 = (heading_size * 0.3).max(16.0);
    let fact_size: f32 = footer_size * 1.25;
    let top: i32 = if heading.is_empty() {
        margin
    } else {
        margin + (heading_size * 1.8) as i32
    };
    let mut bottom: i32 = if footer.is_empty() {
        height - margin
    } else {
        height - margin - (footer_size * 1.5) as i32
    };
    let fact_baseline: i32 = if footer.is_empty() {
        height - margin
    } else {
        bottom
    };
    if !fact.is_empty() {
        bottom = fact_baseline - (fact_size * 1.6) as i32;
    }
    let most: i32 = (short * SPRAY_SHARE) as i32;
    let packed: i32 = (short * PACKED_ROUND_SHARE) as i32;
    let inset: i32 = (short * SPRAY_INSET) as i32;
    let (top_side, bottom_side): (i32, i32) = if packed_round {
        (packed, packed)
    } else {
        (top.min(most), (height - bottom).min(most))
    };
    let placed_top: Option<PlacedSpray> = sprays
        .top_left
        .as_ref()
        .and_then(|spray: &Spray| spray.place((0, 0), top_side, inset, true));
    let placed_bottom: Option<PlacedSpray> = sprays
        .bottom_right
        .as_ref()
        .and_then(|spray: &Spray| spray.place((width, height), bottom_side, inset, false));
    // Words are centred in the span their corner's spray leaves them.
    let heading_span: (i32, i32) = match &placed_top {
        Some(placed) => (
            placed.block.x + placed.block.width + margin / 2,
            width - margin,
        ),
        None => (margin, width - margin),
    };
    let footer_span: (i32, i32) = match &placed_bottom {
        Some(placed) => (margin, placed.block.x - margin / 2),
        None => (margin, width - margin),
    };
    let footer_room: i32 = footer_span.1 - footer_span.0;
    let heading_x: i32 = centre_in(heading_span, width, heading_em * heading_size);
    let footer_x: i32 = centre_in(footer_span, width, 0.0);
    let fact_x: i32 = centre_in(footer_span, width, fact_em * fact_size);
    // The room either side of a line's centre, so a line moved off the span's middle still
    // cannot be drawn into the spray.
    let room = |span: (i32, i32), x: i32| 2 * (x - span.0).min(span.1 - x);
    let heading_room: i32 = room(heading_span, heading_x);
    let fact_room: i32 = room(footer_span, fact_x);
    let mut placed_sprays: Vec<PlacedSpray> = Vec::new();
    for placed in placed_top.into_iter().chain(placed_bottom) {
        page.images.push(placed.block);
        placed_sprays.push(placed);
    }
    if !heading.is_empty() {
        page.text.push(TextBlock {
            text: String::from(heading),
            x: heading_x,
            baseline_y: margin + heading_size as i32,
            size_px: heading_size,
            max_width: Some(heading_room.max(1)),
            align: Align::Centre,
            style: TextStyle::Heading,
        });
    }
    if !footer.is_empty() {
        page.text.push(TextBlock {
            text: String::from(footer),
            x: footer_x,
            baseline_y: height - margin,
            size_px: footer_size,
            max_width: Some(footer_room.max(1)),
            align: Align::Centre,
            style: TextStyle::Footer,
        });
    }
    if !fact.is_empty() {
        page.text.push(TextBlock {
            text: String::from(fact),
            x: fact_x,
            baseline_y: fact_baseline,
            size_px: fact_size,
            max_width: Some(fact_room.max(1)),
            align: Align::Centre,
            style: TextStyle::Footer,
        });
    }
    Body {
        margin,
        width,
        top,
        bottom,
        heading_size,
        footer_size,
        sprays: placed_sprays,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn a_line_sits_mid_page_unless_the_spray_pushes_it_aside() {
        // A page 1000 wide whose spray leaves 50..700.
        let span: (i32, i32) = (50, 700);
        assert_eq!(centre_in(span, 1000, 300.0), 500, "fits in the middle");
        assert_eq!(
            centre_in(span, 1000, 600.0),
            400,
            "moved only as far as needed"
        );
        assert_eq!(
            centre_in(span, 1000, 0.0),
            375,
            "unknown width: the span's middle"
        );
        assert_eq!(
            centre_in(span, 1000, 900.0),
            375,
            "too wide: the span's middle"
        );
    }
}
