//! The year so far: a headline, two short columns, and the year list.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use frame_core::Resolution;

use crate::align::Align;
use crate::corner_sprays::CornerSprays;
use crate::page_frame::{self, AVERAGE_LETTER_WIDTH, Body, Furniture};
use crate::page_layout::PageLayout;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// Most lines in each of the two columns.
const COLUMN_LINES: usize = 7;
/// The smallest the year list is set before it is cut short.
const SMALLEST_LIST: f32 = 18.0;

/// A short titled column of lines, each a main text and a detail on the right.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeasonColumn {
    /// The column's title.
    pub title: String,
    /// Its lines: the main text, and a detail such as a date (empty for none).
    pub lines: Vec<(String, String)>,
}

/// Settings for [`SeasonPage::layout`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeasonOptions {
    /// The heading at the top of the page.
    pub heading: String,
    /// A small line at the foot. Empty for none.
    pub footer: String,
    /// The border, if any.
    pub sprays: CornerSprays,
    /// One large line, e.g. how many species have been heard this year.
    pub headline: String,
    /// The left column, e.g. recent arrivals.
    pub left: SeasonColumn,
    /// The right column, e.g. this week against the same week last year.
    pub right: SeasonColumn,
    /// The title over the year list.
    pub list_title: String,
    /// The year list, in the order to print it.
    pub list: Vec<String>,
    /// The line counting names left off; `{count}` is replaced by the number.
    pub more: String,
}

/// Lays out the season page: the headline, two columns side by side (one above the other on
/// portrait pages), and the year list set as small as it needs to fit, in several columns.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeasonPage;

impl SeasonPage {
    /// Returns the page for `options` at `resolution`.
    #[must_use]
    pub fn layout(resolution: Resolution, options: &SeasonOptions) -> PageLayout {
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
        let headline_size: f32 = body.heading_size * 0.75;
        let mut top: i32 = body.top + headline_size as i32;
        page.text.push(TextBlock {
            text: options.headline.clone(),
            x: body.width / 2,
            baseline_y: top,
            size_px: headline_size,
            max_width: Some(body.width - 2 * body.margin),
            align: Align::Centre,
            style: TextStyle::Name,
        });
        top += (headline_size * 0.7) as i32;
        let line_size: f32 = (body.heading_size * 0.36).max(16.0);
        let landscape: bool = body.width > body.bottom - body.top;
        let inner: i32 = body.width - 2 * body.margin;
        let columns: [&SeasonColumn; 2] = [&options.left, &options.right];
        if landscape {
            let width: i32 = (inner - body.margin) / 2;
            let mut bottom: i32 = top;
            for (index, column) in columns.into_iter().enumerate() {
                let left: i32 =
                    body.margin + i32::try_from(index).unwrap_or(0) * (width + body.margin);
                bottom = bottom.max(push_column(
                    &mut page.text,
                    column,
                    Place {
                        left,
                        top,
                        width,
                        bottom: body.bottom,
                    },
                    line_size,
                ));
            }
            top = bottom;
        } else {
            for column in columns {
                let place = Place {
                    left: body.margin,
                    top,
                    width: inner,
                    bottom: body.bottom,
                };
                top = push_column(&mut page.text, column, place, line_size);
            }
        }
        // The year list needs a title and a row of names above the footer.
        if !options.list.is_empty() && top + (line_size * 3.5) as i32 <= body.bottom {
            top += (line_size * 1.6) as i32;
            page.text
                .push(title(&options.list_title, body.margin, top, line_size));
            push_list(
                &mut page.text,
                options,
                &body,
                top + (line_size * 0.6) as i32,
                landscape,
            );
        }
        page
    }
}

fn title(text: &str, x: i32, baseline_y: i32, size_px: f32) -> TextBlock {
    TextBlock {
        text: String::from(text),
        x,
        baseline_y,
        size_px: size_px * 1.1,
        max_width: None,
        align: Align::Left,
        style: TextStyle::Detail,
    }
}

/// Where a column goes: its left edge, top and width, and the lowest baseline it may use.
#[derive(Clone, Copy, Debug)]
struct Place {
    left: i32,
    top: i32,
    width: i32,
    bottom: i32,
}

/// Adds `column` at `place`, leaving off lines below its bottom, and returns where it ends.
fn push_column(text: &mut Vec<TextBlock>, column: &SeasonColumn, place: Place, size: f32) -> i32 {
    let Place {
        left,
        top,
        width,
        bottom,
    } = place;
    if column.title.is_empty() && column.lines.is_empty() {
        return top;
    }
    let step: i32 = (size * 1.35) as i32;
    let mut baseline: i32 = top + (size * 1.6) as i32;
    if baseline > bottom {
        return top;
    }
    text.push(title(&column.title, left, baseline, size));
    for (main, detail) in column.lines.iter().take(COLUMN_LINES) {
        if baseline + step > bottom {
            break;
        }
        baseline += step;
        let detail_width: i32 = if detail.is_empty() {
            0
        } else {
            ((detail.chars().count() as f32 * AVERAGE_LETTER_WIDTH + 1.5) * size) as i32
        };
        text.push(TextBlock {
            text: main.clone(),
            x: left,
            baseline_y: baseline,
            size_px: size,
            max_width: Some(width - detail_width),
            align: Align::Left,
            style: TextStyle::Name,
        });
        if !detail.is_empty() {
            text.push(TextBlock {
                text: detail.clone(),
                x: left + width,
                baseline_y: baseline,
                size_px: size * 0.85,
                max_width: None,
                align: Align::Right,
                style: TextStyle::Footer,
            });
        }
    }
    baseline
}

fn push_list(
    text: &mut Vec<TextBlock>,
    options: &SeasonOptions,
    body: &Body,
    top: i32,
    landscape: bool,
) {
    let columns: usize = if landscape { 4 } else { 3 };
    let height: i32 = body.bottom - top;
    let inner: i32 = body.width - 2 * body.margin;
    let gutter: i32 = body.margin / 2;
    let columns_i32: i32 = i32::try_from(columns).unwrap_or(1);
    let column_width: i32 = (inner - gutter * (columns_i32 - 1)) / columns_i32;
    let longest: usize = options
        .list
        .iter()
        .map(|name| name.chars().count() + 4)
        .max()
        .unwrap_or(1);
    let fitting_width: f32 = column_width as f32 / (longest as f32 * AVERAGE_LETTER_WIDTH);
    // As large as fits: shrink until every name fits, or the smallest size is reached.
    let rows_for = |size: f32| -> usize { (height as f32 / (size * 1.25)).max(1.0) as usize };
    let mut size: f32 = fitting_width.min(body.heading_size * 0.34);
    while size > SMALLEST_LIST && rows_for(size) * columns < options.list.len() {
        size -= 1.0;
    }
    let rows: usize = rows_for(size);
    let room: usize = rows * columns;
    let (shown, hidden): (usize, usize) = if options.list.len() > room {
        (room - 1, options.list.len() - (room - 1))
    } else {
        (options.list.len(), 0)
    };
    let rows: usize = shown.div_ceil(columns).max(1).min(rows);
    let place = |index: usize| -> (i32, i32) {
        let column: i32 = i32::try_from(index / rows).unwrap_or(0);
        let line: i32 = i32::try_from(index % rows).unwrap_or(0);
        (
            body.margin + column * (column_width + gutter),
            top + ((line + 1) as f32 * size * 1.25) as i32,
        )
    };
    for (index, name) in options.list.iter().take(shown).enumerate() {
        let (x, baseline_y) = place(index);
        let mut line: String = (index + 1).to_string();
        line.push(' ');
        line.push_str(name);
        text.push(TextBlock {
            text: line,
            x,
            baseline_y,
            size_px: size,
            max_width: Some(column_width),
            align: Align::Left,
            style: TextStyle::Name,
        });
    }
    if hidden > 0 {
        let (x, baseline_y) = place(shown);
        text.push(TextBlock {
            text: options.more.replace("{count}", &hidden.to_string()),
            x,
            baseline_y,
            size_px: size,
            max_width: Some(column_width),
            align: Align::Left,
            style: TextStyle::Detail,
        });
    }
}

#[cfg(test)]
mod test {
    use alloc::{format, vec};

    use super::*;

    fn options(names: usize) -> SeasonOptions {
        SeasonOptions {
            heading: String::from("The year so far"),
            footer: String::new(),
            sprays: CornerSprays::default(),
            headline: String::from("87 species this year"),
            left: SeasonColumn {
                title: String::from("Arrivals"),
                lines: vec![(String::from("Fieldfare"), String::from("30 Oct"))],
            },
            right: SeasonColumn {
                title: String::from("This week last year"),
                lines: vec![(String::from("18 species now, 21 then"), String::new())],
            },
            list_title: String::from("Year list"),
            list: (0..names)
                .map(|index| format!("Bird number {index}"))
                .collect(),
            more: String::from("and {count} more"),
        }
    }

    #[test]
    fn numbers_the_year_list_inside_the_page() {
        for resolution in [Resolution::new(1600, 1200), Resolution::new(1200, 1600)] {
            let page = SeasonPage::layout(resolution, &options(60));
            assert!(
                page.text
                    .iter()
                    .any(|block| block.text == "1 Bird number 0")
            );
            assert!(
                page.text
                    .iter()
                    .any(|block| block.text == "60 Bird number 59")
            );
            for block in &page.text {
                assert!(
                    block.baseline_y > 0 && block.baseline_y <= resolution.height as i32,
                    "{block:?}"
                );
            }
        }
    }

    #[test]
    fn cuts_a_long_list_short_on_a_small_page() {
        let page = SeasonPage::layout(Resolution::new(800, 480), &options(200));
        assert!(page.text.iter().any(|block| block.text.ends_with("more")));
        for block in &page.text {
            assert!(block.baseline_y <= 480, "{block:?}");
        }
    }

    #[test]
    fn keeps_small_pages_above_their_footer() {
        for resolution in [
            Resolution::new(296, 128),
            Resolution::new(400, 300),
            Resolution::new(128, 296),
        ] {
            let mut options: SeasonOptions = options(40);
            options.footer = String::from("Updated 12:00");
            let page = SeasonPage::layout(resolution, &options);
            let footer_top: i32 = page
                .text
                .iter()
                .find(|block| block.text == options.footer)
                .map_or(i32::MAX, |block| block.baseline_y - block.size_px as i32);
            for block in page
                .text
                .iter()
                .filter(|block| block.text != options.footer)
            {
                assert!(block.baseline_y <= footer_top, "{resolution:?} {block:?}");
            }
        }
    }
}
