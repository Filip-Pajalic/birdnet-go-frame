//! The illustrated page: a picture of every bird heard, heavier birds bigger.

use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

use frame_core::Resolution;

use crate::align::Align;
use crate::image_block::ImageBlock;
use crate::page_frame::{self, AVERAGE_LETTER_WIDTH, Body};
use crate::page_layout::PageLayout;
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// One bird to place.
#[derive(Clone, Debug, PartialEq)]
pub struct CollageBird {
    /// The name written under the picture.
    pub label: String,
    /// Relative size, e.g. from [`crate::size_weights`]; 1.0 is average.
    pub weight: f32,
    /// The picture's width divided by its height.
    pub aspect_ratio: f32,
}

/// Settings for [`Collage::layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct CollageOptions {
    /// The heading at the top of the page.
    pub heading: String,
    /// A small line at the foot, e.g. when the page was drawn. Empty for none.
    pub footer: String,
    /// Most birds pictured; the rest are counted in one line.
    pub max_birds: usize,
}

impl Default for CollageOptions {
    fn default() -> Self {
        Self {
            heading: String::from("Heard in the garden"),
            footer: String::new(),
            max_birds: 15,
        }
    }
}

/// Lays birds out in rows, as large as the page allows, each with its name underneath.
///
/// Birds are given most important first. They fill rows from the top, every row about as
/// wide as the others, and the number of rows is whichever lets the birds be drawn largest.
/// Within a row the biggest bird sits in the middle, birds stand on a common shelf, and each
/// faces the middle of the page. [`ImageBlock::art`] is the bird's index in the input.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collage;

impl Collage {
    /// Returns the page for `birds` at `resolution`.
    #[must_use]
    pub fn layout(
        birds: &[CollageBird],
        resolution: Resolution,
        options: &CollageOptions,
    ) -> PageLayout {
        let mut page = PageLayout::new(resolution);
        let mut body: Body = page_frame::frame(&mut page, &options.heading, &options.footer);
        if birds.is_empty() {
            page_frame::nothing_heard(&mut page, &body);
            return page;
        }
        let shown: usize = birds.len().min(options.max_birds.max(1));
        let hidden: usize = birds.len() - shown;
        if hidden > 0 {
            let size: f32 = body.footer_size * 1.2;
            page.text.push(TextBlock {
                text: format!("and {hidden} more"),
                x: body.width / 2,
                baseline_y: body.bottom,
                size_px: size,
                max_width: None,
                align: Align::Centre,
                style: TextStyle::Detail,
            });
            body.bottom -= (size * 1.6) as i32;
        }
        let shelf = Shelves::new(&birds[..shown], &body);
        let mut best: Option<(f32, Vec<Range<usize>>)> = None;
        for row_count in 1..=shown {
            let rows: Vec<Range<usize>> = shelf.partition(row_count);
            let scale: f32 = shelf.largest_scale(&rows);
            if best
                .as_ref()
                .is_none_or(|(best_scale, _)| scale > *best_scale)
            {
                best = Some((scale, rows));
            }
        }
        if let Some((scale, rows)) = best {
            shelf.place(&mut page, scale, &rows);
        }
        page
    }
}

/// The birds and the space for them, measured at any scale.
struct Shelves<'a> {
    birds: &'a [CollageBird],
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    page_centre: f32,
    largest_label: f32,
}

/// One bird's measurements at a given scale.
struct Size {
    bird_width: f32,
    bird_height: f32,
    cell_width: f32,
}

impl<'a> Shelves<'a> {
    fn new(birds: &'a [CollageBird], body: &Body) -> Self {
        Self {
            birds,
            left: body.margin as f32,
            top: body.top as f32,
            width: (body.width - 2 * body.margin).max(1) as f32,
            height: (body.bottom - body.top).max(1) as f32,
            page_centre: body.width as f32 / 2.0,
            largest_label: body.heading_size * 0.45,
        }
    }

    /// Name size at `scale`, which is the height of an average bird in pixels.
    fn label_size(&self, scale: f32) -> f32 {
        (scale * 0.16).clamp(14.0, self.largest_label.max(14.0))
    }

    /// Space between birds at `scale`.
    fn gap(scale: f32) -> f32 {
        (scale * 0.1).max(8.0)
    }

    fn size(&self, index: usize, scale: f32) -> Size {
        let bird: &CollageBird = &self.birds[index];
        // Weight sets the square root of the area, so a long tail does not make a bird huge.
        let root_aspect: f32 = libm::sqrtf(bird.aspect_ratio.clamp(0.2, 5.0));
        let weight: f32 = if bird.weight > 0.0 { bird.weight } else { 1.0 };
        let bird_width: f32 = scale * weight * root_aspect;
        let bird_height: f32 = scale * weight / root_aspect;
        // Names may be squeezed a little by the renderer, so they claim three quarters of their
        // natural width.
        let label_width: f32 = bird.label.chars().count() as f32
            * AVERAGE_LETTER_WIDTH
            * self.label_size(scale)
            * 0.75;
        Size {
            bird_width,
            bird_height,
            cell_width: bird_width.max(label_width),
        }
    }

    /// Splits the birds, in order, into `row_count` rows of about equal width.
    fn partition(&self, row_count: usize) -> Vec<Range<usize>> {
        let count: usize = self.birds.len();
        let widths: Vec<f32> = (0..count)
            .map(|index| self.size(index, 1.0).bird_width)
            .collect();
        let total: f32 = widths.iter().sum();
        let mut rows: Vec<Range<usize>> = Vec::with_capacity(row_count);
        let mut start: usize = 0;
        let mut so_far: f32 = 0.0;
        for index in 0..count {
            so_far += widths[index];
            let rows_after: usize = row_count - rows.len() - 1;
            if rows_after == 0 {
                continue;
            }
            let target: f32 = total * (rows.len() + 1) as f32 / row_count as f32;
            let next_half: f32 = widths.get(index + 1).map_or(0.0, |width| width / 2.0);
            if count - index - 1 == rows_after || so_far + next_half >= target {
                rows.push(start..index + 1);
                start = index + 1;
            }
        }
        rows.push(start..count);
        rows
    }

    fn row_height(&self, row: &Range<usize>, scale: f32) -> f32 {
        let tallest: f32 = row
            .clone()
            .map(|index| self.size(index, scale).bird_height)
            .fold(0.0, f32::max);
        tallest + self.label_size(scale) * 1.4
    }

    fn fits(&self, rows: &[Range<usize>], scale: f32) -> bool {
        let gap: f32 = Self::gap(scale);
        let mut total_height: f32 = gap * (rows.len() as f32 - 1.0);
        for row in rows {
            let row_width: f32 = row
                .clone()
                .map(|index| self.size(index, scale).cell_width)
                .sum::<f32>()
                + gap * (row.len() as f32 - 1.0);
            if row_width > self.width {
                return false;
            }
            total_height += self.row_height(row, scale);
        }
        total_height <= self.height
    }

    /// Returns the largest scale at which `rows` fit, found by bisection.
    fn largest_scale(&self, rows: &[Range<usize>]) -> f32 {
        let (mut low, mut high): (f32, f32) = (1.0, self.height);
        if !self.fits(rows, low) {
            return 0.0;
        }
        for _ in 0..32 {
            let middle: f32 = (low + high) / 2.0;
            if self.fits(rows, middle) {
                low = middle;
            } else {
                high = middle;
            }
        }
        low
    }

    fn place(&self, page: &mut PageLayout, scale: f32, rows: &[Range<usize>]) {
        let scale: f32 = scale.max(1.0);
        let gap: f32 = Self::gap(scale);
        let label_size: f32 = self.label_size(scale);
        let used: f32 = rows
            .iter()
            .map(|row| self.row_height(row, scale))
            .sum::<f32>()
            + gap * (rows.len() as f32 - 1.0);
        let spare: f32 = ((self.height - used) / (rows.len() as f32 + 1.0)).max(0.0);
        let mut row_top: f32 = self.top + spare;
        for row in rows {
            let order: Vec<usize> = self.middle_out(row, scale);
            let sizes: Vec<Size> = order.iter().map(|&index| self.size(index, scale)).collect();
            let shelf: f32 = row_top
                + sizes
                    .iter()
                    .map(|size| size.bird_height)
                    .fold(0.0, f32::max);
            let row_width: f32 = sizes.iter().map(|size| size.cell_width).sum();
            let spacing: f32 = (self.width - row_width) / (order.len() as f32 + 1.0);
            let mut x: f32 = self.left + spacing;
            for (&index, size) in order.iter().zip(&sizes) {
                let centre: f32 = x + size.cell_width / 2.0;
                page.images.push(ImageBlock {
                    art: index,
                    x: (centre - size.bird_width / 2.0) as i32,
                    y: (shelf - size.bird_height) as i32,
                    width: size.bird_width as i32,
                    height: size.bird_height as i32,
                    mirrored: centre > self.page_centre + 1.0,
                });
                page.text.push(TextBlock {
                    text: self.birds[index].label.clone(),
                    x: centre as i32,
                    baseline_y: (shelf + label_size * 1.1) as i32,
                    size_px: label_size,
                    max_width: Some((size.cell_width + spacing * 0.8) as i32),
                    align: Align::Centre,
                    style: TextStyle::Name,
                });
                x += size.cell_width + spacing;
            }
            row_top += self.row_height(row, scale) + gap + spare;
        }
    }

    /// Returns the row's birds with the tallest in the middle and smaller ones outwards.
    fn middle_out(&self, row: &Range<usize>, scale: f32) -> Vec<usize> {
        let mut by_height: Vec<usize> = row.clone().collect();
        by_height.sort_by(|&left, &right| {
            self.size(right, scale)
                .bird_height
                .total_cmp(&self.size(left, scale).bird_height)
                .then(left.cmp(&right))
        });
        let mut arranged: VecDeque<usize> = VecDeque::with_capacity(by_height.len());
        for (position, index) in by_height.into_iter().enumerate() {
            if position % 2 == 0 {
                arranged.push_back(index);
            } else {
                arranged.push_front(index);
            }
        }
        arranged.into_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn bird(label: &str, weight: f32, aspect_ratio: f32) -> CollageBird {
        CollageBird {
            label: String::from(label),
            weight,
            aspect_ratio,
        }
    }

    fn garden(count: usize) -> Vec<CollageBird> {
        (0..count)
            .map(|index| {
                bird(
                    "Great Spotted Woodpecker",
                    0.7 + (index % 5) as f32 * 0.15,
                    0.8 + (index % 3) as f32 * 0.4,
                )
            })
            .collect()
    }

    fn overlaps(first: &ImageBlock, second: &ImageBlock) -> bool {
        first.x < second.x + second.width
            && second.x < first.x + first.width
            && first.y < second.y + second.height
            && second.y < first.y + first.height
    }

    #[test]
    fn empty_page_says_so() {
        let page = Collage::layout(&[], Resolution::new(1600, 1200), &CollageOptions::default());
        assert!(page.images.is_empty());
        assert!(
            page.text
                .iter()
                .any(|block| block.text == "No birds heard yet")
        );
    }

    #[test]
    fn birds_stay_inside_the_page_and_apart() {
        for resolution in [Resolution::new(1600, 1200), Resolution::new(1200, 1600)] {
            for count in [1, 2, 5, 9, 15, 30] {
                let page = Collage::layout(&garden(count), resolution, &CollageOptions::default());
                assert_eq!(page.images.len(), count.min(15));
                for (position, image) in page.images.iter().enumerate() {
                    assert!(image.x >= 0 && image.y >= 0, "{image:?}");
                    assert!(
                        image.x + image.width <= i32::try_from(resolution.width).expect("small")
                    );
                    assert!(
                        image.y + image.height <= i32::try_from(resolution.height).expect("small")
                    );
                    assert!(image.width > 40, "{count} birds: {image:?}");
                    for other in &page.images[position + 1..] {
                        assert!(!overlaps(image, other), "{image:?} {other:?}");
                    }
                }
                let mut arts: Vec<usize> = page.images.iter().map(|image| image.art).collect();
                arts.sort_unstable();
                assert_eq!(arts, (0..count.min(15)).collect::<Vec<usize>>());
                assert_eq!(
                    page.text.iter().any(|block| block.text == "and 15 more"),
                    count == 30
                );
            }
        }
    }

    #[test]
    fn heavier_birds_are_drawn_bigger_and_face_the_middle() {
        let page = Collage::layout(
            &[
                bird("Wren", 0.7, 1.0),
                bird("Pigeon", 1.4, 1.0),
                bird("Robin", 1.0, 1.0),
            ],
            Resolution::new(1600, 1200),
            &CollageOptions::default(),
        );
        let image = |art: usize| -> ImageBlock {
            *page
                .images
                .iter()
                .find(|image| image.art == art)
                .expect("placed")
        };
        assert!(image(1).height > image(2).height && image(2).height > image(0).height);
        for placed in &page.images {
            let centre: i32 = placed.x + placed.width / 2;
            assert_eq!(placed.mirrored, centre > 801, "{placed:?}");
        }
        let names: Vec<&str> = page.text.iter().map(|block| block.text.as_str()).collect();
        assert!(names.contains(&"Wren") && names.contains(&"Pigeon"));
    }

    #[test]
    fn rows_are_balanced() {
        let birds: Vec<CollageBird> = (0..6).map(|_| bird("Tit", 1.0, 1.0)).collect();
        let body = Body {
            margin: 0,
            width: 1000,
            top: 0,
            bottom: 1000,
            heading_size: 40.0,
            footer_size: 16.0,
        };
        let shelves = Shelves::new(&birds, &body);
        assert_eq!(shelves.partition(2), [0..3, 3..6]);
        assert_eq!(shelves.partition(3), [0..2, 2..4, 4..6]);
        assert_eq!(shelves.partition(6).len(), 6);
    }
}
