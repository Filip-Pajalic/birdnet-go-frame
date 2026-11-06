//! The day's story: a few lines on the day, and a chart of when each bird sang.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use frame_core::{Ink, Resolution};

use crate::align::Align;
use crate::corner_sprays::CornerSprays;
use crate::page_frame::{self, AVERAGE_LETTER_WIDTH, Body, Furniture};
use crate::page_layout::PageLayout;
use crate::shape_block::ShapeBlock;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// The fewest hours the chart spans, so a quiet morning is not stretched across the page.
const FEWEST_HOURS: usize = 10;

/// One row of the chorus chart: a name and its detections hour by hour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChorusRow {
    /// The name written beside the row.
    pub label: String,
    /// Detections in each local hour, index 0 for midnight to one.
    pub hours: [u32; 24],
    /// The ink the row's bars are engraved in.
    pub ink: Ink,
    /// Whether the bars are hatched close, darker, as for the whole chorus.
    pub heavy: bool,
}

/// A moment marked across the chart, such as sunrise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChartMark {
    /// Minutes after local midnight.
    pub minute: u32,
    /// The words written at the top of the line.
    pub label: String,
}

/// Settings for [`StoryPage::layout`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryOptions {
    /// The heading at the top of the page.
    pub heading: String,
    /// A small line at the foot. Empty for none.
    pub footer: String,
    /// The border, if any.
    pub sprays: CornerSprays,
    /// Lines about the day, e.g. the first and last singers and the weather.
    pub lines: Vec<String>,
    /// The chart's rows, top to bottom.
    pub rows: Vec<ChorusRow>,
    /// Moments marked across the chart.
    pub marks: Vec<ChartMark>,
    /// The line shown when nothing has been heard.
    pub empty: String,
}

/// Lays out the day's story: its lines at the top, then one row per species showing when it
/// sang, as bars hour by hour over the hours anything sang, scaled per row so a quiet bird's
/// pattern shows as well as a loud one's. Rows that do not fit at a legible size are left off,
/// from the last.
#[derive(Clone, Copy, Debug, Default)]
pub struct StoryPage;

impl StoryPage {
    /// Returns the page for `options` at `resolution`.
    #[must_use]
    pub fn layout(resolution: Resolution, options: &StoryOptions) -> PageLayout {
        let mut page = PageLayout::new(resolution);
        let body: Body = page_frame::frame(
            &mut page,
            &Furniture {
                heading: &options.heading,
                fact: "",
                footer: &options.footer,
                sprays: &options.sprays,
                packed_round: false,
                widths_em: (0.0, 0.0),
            },
        );
        let line_size: f32 = body.heading_size * 0.5;
        let mut top: i32 = body.top;
        // Lines that would reach into the footer are left off, the last first.
        for line in &options.lines {
            let baseline: i32 = top + (line_size * 1.45) as i32;
            if baseline > body.bottom {
                break;
            }
            top = baseline;
            page.text.push(TextBlock {
                text: line.clone(),
                x: body.margin,
                baseline_y: top,
                size_px: line_size,
                max_width: Some(body.width - 2 * body.margin),
                align: Align::Left,
                style: TextStyle::Name,
            });
        }
        let chart = Body {
            top: top + (line_size * 1.2) as i32,
            ..body
        };
        if options
            .rows
            .iter()
            .all(|row| row.hours.iter().all(|&count| count == 0))
        {
            page_frame::nothing_heard(&mut page, &chart, &options.empty);
        } else {
            push_chart(&mut page, &chart, options);
        }
        page
    }
}

/// Returns the first hour and the number of hours the chart spans.
fn span(options: &StoryOptions) -> (usize, usize) {
    let active = |hour: &usize| options.rows.iter().any(|row| row.hours[*hour] > 0);
    let first: usize = (0..24).find(active).unwrap_or(0);
    let last: usize = (0..24).rev().find(active).unwrap_or(23);
    let marked = options
        .marks
        .iter()
        .map(|mark| (mark.minute / 60).min(23) as usize);
    let first: usize = marked.clone().fold(first, usize::min);
    let last: usize = marked.fold(last, usize::max);
    let hours: usize = (last + 1 - first).clamp(FEWEST_HOURS, 24);
    (first.min(24 - hours), hours)
}

fn push_chart(page: &mut PageLayout, body: &Body, options: &StoryOptions) {
    let (first_hour, hours): (usize, usize) = span(options);
    let tick_size: f32 = (body.heading_size * 0.3).max(16.0);
    let chart_top: i32 = body.top + (tick_size * 1.4) as i32;
    let chart_bottom: i32 = body.bottom - (tick_size * 1.6) as i32;
    // A page too small for one legible row gets no chart rather than one over the footer.
    if chart_bottom - chart_top < tick_size as i32 {
        return;
    }
    // Rows below a legible height are left off, the last first.
    let fitting: usize = usize::try_from((chart_bottom - chart_top) / (tick_size * 1.7) as i32)
        .unwrap_or(1)
        .max(1);
    let shown: &[ChorusRow] = &options.rows[..options.rows.len().min(fitting)];
    let rows: i32 = i32::try_from(shown.len()).unwrap_or(1).max(1);
    let row_height: i32 = ((chart_bottom - chart_top) / rows).max(8);
    let longest: usize = shown
        .iter()
        .map(|row| row.label.chars().count())
        .max()
        .unwrap_or(1);
    // On a small page the heading is so small that 14 px is already the larger bound.
    let label_size: f32 =
        (row_height as f32 * 0.5).clamp(14.0, (body.heading_size * 0.42).max(14.0));
    let label_width: i32 = ((longest as f32 * AVERAGE_LETTER_WIDTH + 1.0) * label_size) as i32;
    let label_width: i32 = label_width.min((body.width - 2 * body.margin) * 2 / 5);
    let chart_left: i32 = body.margin + label_width;
    let chart_width: i32 = body.width - body.margin - chart_left;
    let column: f32 = chart_width as f32 / hours as f32;
    let bar: i32 = ((column * 0.72) as i32).max(2);
    // Hairlines, as an engraver's: thicker only on big pages, so they survive dithering.
    let hair: i32 = ((body.heading_size / 45.0) as i32).max(1);
    // The left edge of the chart, like the ruled margin of a table in an old natural history.
    page.shapes.push(ShapeBlock {
        x: chart_left - hair * 3,
        y: chart_top,
        width: hair,
        height: rows * row_height,
        ink: Ink::Black,
    });

    for (index, row) in shown.iter().enumerate() {
        let index: i32 = i32::try_from(index).unwrap_or(0);
        let baseline: i32 = chart_top + (index + 1) * row_height - row_height / 8;
        page.text.push(TextBlock {
            text: row.label.clone(),
            x: body.margin,
            baseline_y: baseline - row_height / 8,
            size_px: label_size,
            max_width: Some(label_width - (label_size * 0.6) as i32),
            align: Align::Left,
            style: TextStyle::Name,
        });
        page.shapes.push(ShapeBlock {
            x: chart_left - hair * 3,
            y: baseline,
            width: chart_width + hair * 3,
            height: hair,
            ink: Ink::Black,
        });
        let loudest: u32 = row.hours.iter().copied().max().unwrap_or(0).max(1);
        let tallest: i32 = row_height * 3 / 4;
        for (offset, &count) in row.hours[first_hour..first_hour + hours].iter().enumerate() {
            if count == 0 {
                continue;
            }
            let height: i32 = (i64::from(tallest) * i64::from(count) / i64::from(loudest))
                .try_into()
                .unwrap_or(tallest)
                .max(hair * 3);
            let left: i32 =
                chart_left + (offset as f32 * column + (column - bar as f32) / 2.0) as i32;
            let bar_box = ShapeBlock {
                x: left,
                y: baseline - height,
                width: bar,
                height,
                ink: row.ink,
            };
            push_engraved(&mut page.shapes, bar_box, hair, row.heavy);
        }
    }

    let chart_end: i32 = chart_top + rows * row_height;
    for mark in &options.marks {
        let hour: f32 = mark.minute as f32 / 60.0 - first_hour as f32;
        if !(0.0..=hours as f32).contains(&hour) {
            continue;
        }
        let x: i32 = chart_left + (hour * column) as i32;
        // A dotted hairline, as engravers marked a reference line.
        let dot: i32 = hair * 2;
        let mut y: i32 = chart_top;
        while y < chart_end {
            page.shapes.push(ShapeBlock {
                x: x - hair / 2,
                y,
                width: hair,
                height: dot.min(chart_end - y),
                ink: Ink::Black,
            });
            y += dot * 3;
        }
        // Centred over its line, unless that would run off the page.
        let half: i32 =
            (mark.label.chars().count() as f32 * AVERAGE_LETTER_WIDTH * tick_size / 2.0) as i32;
        let align: Align = if x + half > body.width - body.margin / 4 {
            Align::Right
        } else if x - half < body.margin / 4 {
            Align::Left
        } else {
            Align::Centre
        };
        page.text.push(TextBlock {
            text: mark.label.clone(),
            x,
            baseline_y: chart_top - (tick_size * 0.4) as i32,
            size_px: tick_size,
            max_width: None,
            align,
            style: TextStyle::Detail,
        });
    }

    // Every hour on wide charts, every other or third on narrow ones.
    let step: usize = if column > tick_size * 2.2 {
        1
    } else if column * 2.0 > tick_size * 2.2 {
        2
    } else {
        3
    };
    for hour in (first_hour..=first_hour + hours).filter(|hour| hour % step == 0) {
        let offset: i32 = i32::try_from(hour - first_hour).unwrap_or(0);
        page.shapes.push(ShapeBlock {
            x: chart_left + (offset as f32 * column) as i32 - hair / 2,
            y: chart_end,
            width: hair,
            height: hair * 4,
            ink: Ink::Black,
        });
        page.text.push(TextBlock {
            text: format!("{:02}", hour % 24),
            x: chart_left + (offset as f32 * column) as i32,
            baseline_y: chart_end + (tick_size * 1.3) as i32,
            size_px: tick_size,
            max_width: None,
            align: Align::Centre,
            style: TextStyle::Footer,
        });
    }
}

/// Adds a bar engraved in `bar.ink`: outlined in hairlines and hatched with upright lines, close
/// when `heavy`, the way a plate's engraver shaded a block.
fn push_engraved(shapes: &mut Vec<ShapeBlock>, bar: ShapeBlock, hair: i32, heavy: bool) {
    if bar.width <= hair * 4 {
        shapes.push(bar);
        return;
    }
    for (x, width) in [(bar.x, hair), (bar.x + bar.width - hair, hair)] {
        shapes.push(ShapeBlock { x, width, ..bar });
    }
    shapes.push(ShapeBlock {
        height: hair,
        ..bar
    });
    let step: i32 = if heavy { hair * 2 } else { hair * 3 };
    let mut x: i32 = bar.x + step;
    while x + hair < bar.x + bar.width - hair {
        shapes.push(ShapeBlock {
            x,
            width: hair,
            ..bar
        });
        x += step;
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    fn options(rows: Vec<ChorusRow>) -> StoryOptions {
        StoryOptions {
            heading: String::from("The day so far"),
            footer: String::new(),
            sprays: CornerSprays::default(),
            lines: vec![String::from("First song 05:05, Eurasian Blackbird")],
            rows,
            marks: vec![ChartMark {
                minute: 7 * 60 + 19,
                label: String::from("Sunrise"),
            }],
            empty: String::from("No birds heard yet"),
        }
    }

    fn row(label: &str, hours: &[(usize, u32)]) -> ChorusRow {
        let mut counts: [u32; 24] = [0; 24];
        for &(hour, count) in hours {
            counts[hour] = count;
        }
        ChorusRow {
            label: String::from(label),
            hours: counts,
            ink: Ink::Green,
            heavy: false,
        }
    }

    #[test]
    fn spans_the_hours_anything_sang() {
        let rows = vec![
            row("Robin", &[(5, 4), (6, 9)]),
            row("Tit", &[(8, 2), (16, 1)]),
        ];
        assert_eq!(span(&options(rows)), (5, 12));
        // A quiet morning still spans ten hours.
        assert_eq!(span(&options(vec![row("Robin", &[(22, 1)])])), (7, 16));
    }

    #[test]
    fn keeps_small_pages_above_their_footer() {
        let rows = vec![row("Robin", &[(5, 4), (6, 9)])];
        for resolution in [
            Resolution::new(64, 64),
            Resolution::new(296, 128),
            Resolution::new(400, 300),
        ] {
            let mut options: StoryOptions = options(rows.clone());
            options.footer = String::from("Updated 12:00");
            options
                .lines
                .push(String::from("The chorus peaked at 06:00"));
            options.lines.push(String::from("Clear, 8°"));
            let page = StoryPage::layout(resolution, &options);
            let footer_top: i32 = page
                .text
                .iter()
                .find(|text| text.text == options.footer)
                .map_or(i32::MAX, |text| text.baseline_y - text.size_px as i32);
            for text in page.text.iter().filter(|text| text.text != options.footer) {
                assert!(text.baseline_y <= footer_top, "{resolution:?} {text:?}");
            }
            for shape in &page.shapes {
                assert!(
                    shape.y + shape.height <= footer_top,
                    "{resolution:?} {shape:?}"
                );
            }
        }
    }

    #[test]
    fn draws_a_bar_per_hour_heard_inside_the_page() {
        let rows = vec![
            row("Robin", &[(5, 4), (6, 9)]),
            row("Tit", &[(8, 2), (16, 1)]),
        ];
        for resolution in [Resolution::new(1600, 1200), Resolution::new(480, 800)] {
            let page = StoryPage::layout(resolution, &options(rows.clone()));
            // Each bar has one horizontal edge, its top; the hatching and sides are upright.
            let bars: usize = page
                .shapes
                .iter()
                .filter(|shape| shape.ink == Ink::Green && shape.width > shape.height)
                .count();
            assert_eq!(bars, 4);
            for shape in &page.shapes {
                assert!(
                    shape.x >= 0 && shape.x + shape.width <= resolution.width as i32,
                    "{shape:?}"
                );
                assert!(
                    shape.y >= 0 && shape.y + shape.height <= resolution.height as i32,
                    "{shape:?}"
                );
            }
            assert!(page.text.iter().any(|block| block.text == "Sunrise"));
        }
    }

    #[test]
    fn leaves_off_rows_that_do_not_fit() {
        let rows: Vec<ChorusRow> = (0..30).map(|_| row("Robin", &[(5, 4)])).collect();
        let page = StoryPage::layout(Resolution::new(800, 480), &options(rows));
        let drawn: usize = page
            .text
            .iter()
            .filter(|block| block.text == "Robin")
            .count();
        assert!((3..30).contains(&drawn), "{drawn}");
    }

    #[test]
    fn a_silent_day_says_so() {
        let page = StoryPage::layout(Resolution::new(800, 480), &options(vec![row("Robin", &[])]));
        assert!(
            page.text
                .iter()
                .any(|block| block.text == "No birds heard yet")
        );
        assert!(page.shapes.is_empty());
    }
}
