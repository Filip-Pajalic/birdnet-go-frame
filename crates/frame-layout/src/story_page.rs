//! The day's story: a few lines on the day, and a chart of when each bird sang.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use frame_core::{Ink, Resolution};

use crate::align::Align;
use crate::page_frame::{self, AVERAGE_LETTER_WIDTH, Body};
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
    /// The ink the row's bars are drawn in.
    pub ink: Ink,
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
/// pattern shows as well as a loud one's.
#[derive(Clone, Copy, Debug, Default)]
pub struct StoryPage;

impl StoryPage {
    /// Returns the page for `options` at `resolution`.
    #[must_use]
    pub fn layout(resolution: Resolution, options: &StoryOptions) -> PageLayout {
        let mut page = PageLayout::new(resolution);
        let body: Body = page_frame::frame(&mut page, &options.heading, &options.footer);
        let line_size: f32 = body.heading_size * 0.5;
        let mut top: i32 = body.top;
        for line in &options.lines {
            top += (line_size * 1.45) as i32;
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
    let rows: i32 = i32::try_from(options.rows.len()).unwrap_or(1).max(1);
    let row_height: i32 = ((chart_bottom - chart_top) / rows).max(8);
    let longest: usize = options
        .rows
        .iter()
        .map(|row| row.label.chars().count())
        .max()
        .unwrap_or(1);
    let label_size: f32 = (row_height as f32 * 0.5).clamp(14.0, body.heading_size * 0.42);
    let label_width: i32 = ((longest as f32 * AVERAGE_LETTER_WIDTH + 1.0) * label_size) as i32;
    let label_width: i32 = label_width.min((body.width - 2 * body.margin) * 2 / 5);
    let chart_left: i32 = body.margin + label_width;
    let chart_width: i32 = body.width - body.margin - chart_left;
    let column: f32 = chart_width as f32 / hours as f32;
    let bar: i32 = ((column * 0.72) as i32).max(2);
    let rule: i32 = (row_height / 40).max(2);

    for (index, row) in options.rows.iter().enumerate() {
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
            x: chart_left,
            y: baseline,
            width: chart_width,
            height: rule,
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
                .max(rule * 2);
            let left: i32 =
                chart_left + (offset as f32 * column + (column - bar as f32) / 2.0) as i32;
            page.shapes.push(ShapeBlock {
                x: left,
                y: baseline - height,
                width: bar,
                height,
                ink: row.ink,
            });
        }
    }

    let chart_end: i32 = chart_top + rows * row_height;
    for mark in &options.marks {
        let hour: f32 = mark.minute as f32 / 60.0 - first_hour as f32;
        if !(0.0..=hours as f32).contains(&hour) {
            continue;
        }
        let x: i32 = chart_left + (hour * column) as i32;
        let dash: i32 = (row_height / 6).max(6);
        let mut y: i32 = chart_top;
        while y < chart_end {
            page.shapes.push(ShapeBlock {
                x: x - rule / 2,
                y,
                width: rule,
                height: dash.min(chart_end - y),
                ink: Ink::Blue,
            });
            y += dash * 2;
        }
        page.text.push(TextBlock {
            text: mark.label.clone(),
            x,
            baseline_y: chart_top - (tick_size * 0.4) as i32,
            size_px: tick_size,
            max_width: None,
            align: Align::Centre,
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

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    fn options(rows: Vec<ChorusRow>) -> StoryOptions {
        StoryOptions {
            heading: String::from("The day so far"),
            footer: String::new(),
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
    fn draws_a_bar_per_hour_heard_inside_the_page() {
        let rows = vec![
            row("Robin", &[(5, 4), (6, 9)]),
            row("Tit", &[(8, 2), (16, 1)]),
        ];
        for resolution in [Resolution::new(1600, 1200), Resolution::new(480, 800)] {
            let page = StoryPage::layout(resolution, &options(rows.clone()));
            let bars: usize = page
                .shapes
                .iter()
                .filter(|shape| shape.ink == Ink::Green)
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
