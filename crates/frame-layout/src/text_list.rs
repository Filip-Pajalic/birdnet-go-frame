//! The text-only page: a heading and a ranked list of species.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use birdnet_model::SpeciesSummary;
use frame_core::Resolution;

use crate::align::Align;
use crate::page_frame::{self, AVERAGE_LETTER_WIDTH, Body};
use crate::page_layout::PageLayout;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// Settings for [`TextList::layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct TextListOptions {
    /// The heading at the top of the page.
    pub heading: String,
    /// A small line at the foot, e.g. when the page was drawn. Empty for none.
    pub footer: String,
    /// Most species shown; the rest are summarised in one line.
    pub max_species: usize,
    /// The line shown when nothing has been heard.
    pub empty: String,
    /// The line counting species left off; `{count}` is replaced by the number.
    pub more: String,
}

impl Default for TextListOptions {
    fn default() -> Self {
        Self {
            heading: String::from("Heard in the garden"),
            footer: String::new(),
            max_species: 24,
            empty: String::from("No birds heard yet"),
            more: String::from("and {count} more"),
        }
    }
}

/// Lays out a heading and the species heard, most detections first, in one or two columns.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextList;

impl TextList {
    /// Returns the page for `species` (already sorted) at `resolution`.
    #[must_use]
    pub fn layout(
        species: &[SpeciesSummary],
        resolution: Resolution,
        options: &TextListOptions,
    ) -> PageLayout {
        let mut page = PageLayout::new(resolution);
        let body: Body = page_frame::frame(&mut page, &options.heading, &options.footer);
        if species.is_empty() {
            page_frame::nothing_heard(&mut page, &body, &options.empty);
        } else {
            push_species(&mut page.text, species, options, &body);
        }
        page
    }
}

fn push_species(
    text: &mut Vec<TextBlock>,
    species: &[SpeciesSummary],
    options: &TextListOptions,
    body: &Body,
) {
    let (max_species, more): (usize, &str) = (options.max_species, &options.more);
    let (margin, width, list_top, list_bottom): (i32, i32, i32, i32) =
        (body.margin, body.width, body.top, body.bottom);
    let shown: usize = species.len().min(max_species.max(1));
    let hidden: usize = species.len() - shown;
    let lines: usize = shown + usize::from(hidden > 0);
    // Long lists on landscape pages use two columns; portrait pages keep one.
    let landscape: bool = width > list_bottom;
    let columns: usize = if lines > 12 && landscape { 2 } else { 1 };
    let rows: usize = lines.div_ceil(columns);
    let row_height: i32 = (list_bottom - list_top) / i32::try_from(rows).unwrap_or(1).max(1);
    let gutter: i32 = margin;
    let columns_i32: i32 = i32::try_from(columns).unwrap_or(1);
    let column_width: i32 = (width - 2 * margin - gutter * (columns_i32 - 1)) / columns_i32;
    // One size for every name, small enough that the longest fits beside its count. The
    // renderer still shrinks any name the estimate gets wrong.
    let longest: usize = species
        .iter()
        .take(shown)
        .map(|row| row.common_name.chars().count())
        .max()
        .unwrap_or(1);
    let fitting: f32 = column_width as f32 / (longest as f32 * AVERAGE_LETTER_WIDTH + 2.4);
    let size: f32 = (row_height as f32 * 0.62).min(fitting).clamp(18.0, 96.0);

    for (index, row) in species.iter().take(shown).enumerate() {
        let column: i32 = i32::try_from(index / rows).unwrap_or(0);
        let line: i32 = i32::try_from(index % rows).unwrap_or(0);
        let left: i32 = margin + column * (column_width + gutter);
        let baseline: i32 = list_top + line * row_height + (size * 1.05) as i32;
        let name: String = if row.common_name.is_empty() {
            row.scientific_name.to_string()
        } else {
            row.common_name.clone()
        };
        text.push(TextBlock {
            text: name,
            x: left,
            baseline_y: baseline,
            size_px: size,
            // Leave room for the count on the right.
            max_width: Some(column_width - (size * 2.4) as i32),
            align: Align::Left,
            style: TextStyle::Name,
        });
        text.push(TextBlock {
            text: format!("{}", row.count),
            x: left + column_width,
            baseline_y: baseline,
            size_px: size * 0.8,
            max_width: None,
            align: Align::Right,
            style: TextStyle::Detail,
        });
    }

    if hidden > 0 {
        let column: i32 = i32::try_from(shown / rows).unwrap_or(0);
        let line: i32 = i32::try_from(shown % rows).unwrap_or(0);
        text.push(TextBlock {
            text: more.replace("{count}", &hidden.to_string()),
            x: margin + column * (column_width + gutter),
            baseline_y: list_top + line * row_height + (size * 1.05) as i32,
            size_px: size * 0.8,
            max_width: None,
            align: Align::Left,
            style: TextStyle::Detail,
        });
    }
}

#[cfg(test)]
mod test {
    use birdnet_model::ScientificName;

    use super::*;

    fn bird(name: &str, count: u32) -> SpeciesSummary {
        SpeciesSummary {
            scientific_name: ScientificName::new("Parus major"),
            common_name: String::from(name),
            count,
            first_heard: None,
            last_heard: None,
            max_confidence: None,
        }
    }

    #[test]
    fn empty_page_says_so() {
        let page = TextList::layout(
            &[],
            Resolution::new(1600, 1200),
            &TextListOptions::default(),
        );
        assert!(
            page.text
                .iter()
                .any(|block| block.text == "No birds heard yet")
        );
    }

    #[test]
    fn every_block_starts_inside_the_page() {
        let species: Vec<SpeciesSummary> =
            (0..40).map(|index| bird("Great Tit", 40 - index)).collect();
        for resolution in [Resolution::new(1600, 1200), Resolution::new(1200, 1600)] {
            let page = TextList::layout(&species, resolution, &TextListOptions::default());
            for block in &page.text {
                assert!(
                    block.baseline_y > 0 && block.baseline_y <= 1600,
                    "{block:?}"
                );
                assert!(block.x >= 0 && block.x <= 1600, "{block:?}");
            }
            assert!(page.text.iter().any(|block| block.text == "and 16 more"));
        }
    }

    #[test]
    fn lists_names_with_counts() {
        let page = TextList::layout(
            &[bird("Great Tit", 45), bird("European Robin", 17)],
            Resolution::new(1600, 1200),
            &TextListOptions::default(),
        );
        let words: Vec<&str> = page.text.iter().map(|block| block.text.as_str()).collect();
        assert_eq!(
            words,
            [
                "Heard in the garden",
                "Great Tit",
                "45",
                "European Robin",
                "17"
            ]
        );
    }
}
