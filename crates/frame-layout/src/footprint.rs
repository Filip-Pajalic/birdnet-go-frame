//! The space one bird and its name take on the page.

use alloc::vec;
use alloc::vec::Vec;

/// How far opaque outlines are shrunk before packing, so neighbours nestle into each other's
/// feathered paper halos, which are invisible on the page. Their bodies still cannot touch.
const OVERLAP: usize = 2;

/// A grid of the pixels a bird (and, once joined, its name) keeps clear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Footprint {
    pub(crate) width: usize,
    pub(crate) height: usize,
    cells: Vec<bool>,
}

/// Where the picture and its name sit inside a joined [`Footprint`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Anchors {
    pub(crate) art: (usize, usize),
    /// The name's box: left, top, width.
    pub(crate) label: Option<(usize, usize, usize)>,
}

impl Footprint {
    /// Wraps an opaque-pixel mask, shrunk by [`OVERLAP`]. A bird thinner than that keeps its
    /// whole mask, or it would reserve nothing and be packed over.
    pub(crate) fn of_art(width: usize, height: usize, opaque: Vec<bool>) -> Self {
        let eroded: Vec<bool> = erode(&opaque, width, height, OVERLAP);
        let cells: Vec<bool> = if eroded.contains(&true) {
            eroded
        } else {
            opaque
        };
        Self {
            width,
            height,
            cells,
        }
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.width + x]
    }

    /// Joins a name `label_width` by `label_height` under the bird, centred on the middle of
    /// its body rather than its box (so a long tail does not pull the name aside), and raised
    /// until it clears the outline: into the gap beside a leg or under a perch. `gap` pixels
    /// are kept between them. Reserving the name with its bird is what guarantees it a place:
    /// the packer leaves no free paper between birds for a name placed afterwards.
    pub(crate) fn with_label(
        &self,
        label_width: usize,
        label_height: usize,
        gap: usize,
    ) -> (Self, Anchors) {
        let mut column_total: usize = 0;
        let mut count: usize = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) {
                    column_total += x;
                    count += 1;
                }
            }
        }
        let centre: f32 = if count == 0 {
            self.width as f32 / 2.0
        } else {
            column_total as f32 / count as f32
        };
        let offset: i64 = libm::roundf(centre - label_width as f32 / 2.0) as i64;
        let left: i64 = offset.min(0);
        let art_x: usize = usize::try_from(-left).unwrap_or(0);
        let label_x: usize = usize::try_from(offset - left).unwrap_or(0);
        let width: usize = self
            .width
            .max(label_x + label_width)
            .max(art_x + self.width);

        // The lowest row of the outline above the name's columns.
        let first: usize = label_x.saturating_sub(art_x);
        let last: usize = (label_x + label_width)
            .saturating_sub(art_x)
            .min(self.width);
        let lowest: Option<usize> = (0..self.height)
            .rev()
            .find(|&y| (first..last).any(|x| self.get(x, y)));
        let label_y: usize = lowest.map_or(self.height, |y| y + 1) + gap;
        let height: usize = self.height.max(label_y + label_height);

        let mut cells: Vec<bool> = vec![false; width * height];
        for y in 0..self.height {
            for x in 0..self.width {
                cells[y * width + art_x + x] = self.get(x, y);
            }
        }
        for y in label_y..label_y + label_height {
            cells[y * width + label_x..y * width + label_x + label_width].fill(true);
        }
        let joined = Self {
            width,
            height,
            cells,
        };
        let anchors = Anchors {
            art: (art_x, 0),
            label: Some((label_x, label_y, label_width)),
        };
        (joined, anchors)
    }

    /// Returns each row's runs of set cells, as start and end columns.
    pub(crate) fn runs(&self) -> Vec<Vec<(usize, usize)>> {
        (0..self.height)
            .map(|y| {
                let row: &[bool] = &self.cells[y * self.width..(y + 1) * self.width];
                let mut runs: Vec<(usize, usize)> = Vec::new();
                let mut start: Option<usize> = None;
                for (x, &set) in row.iter().enumerate() {
                    match (set, start) {
                        (true, None) => start = Some(x),
                        (false, Some(begin)) => {
                            runs.push((begin, x));
                            start = None;
                        }
                        _ => {}
                    }
                }
                if let Some(begin) = start {
                    runs.push((begin, row.len()));
                }
                runs
            })
            .collect()
    }
}

/// Keeps a cell only if every cell within `radius` (a square) is set; beyond the edge counts
/// as set, so outlines touching the edge are not eaten away.
fn erode(cells: &[bool], width: usize, height: usize, radius: usize) -> Vec<bool> {
    let across: Vec<bool> = (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let (left, right) = (x.saturating_sub(radius), (x + radius).min(width - 1));
            cells[y * width + left..=y * width + right]
                .iter()
                .all(|&set| set)
        })
        .collect();
    (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let (top, bottom) = (y.saturating_sub(radius), (y + radius).min(height - 1));
            (top..=bottom).all(|row| across[row * width + x])
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    fn block(width: usize, height: usize, solid: impl Fn(usize, usize) -> bool) -> Footprint {
        let cells: Vec<bool> = (0..width * height)
            .map(|index| solid(index % width, index / width))
            .collect();
        Footprint::of_art(width, height, cells)
    }

    #[test]
    fn erodes_but_never_to_nothing() {
        let wide = block(20, 20, |x, y| (5..15).contains(&x) && (5..15).contains(&y));
        assert!(!wide.get(5, 10) && wide.get(7, 10));
        let thin = block(20, 20, |x, _| x == 10);
        assert!(thin.get(10, 3), "a one-pixel line keeps its mask");
    }

    #[test]
    fn tucks_the_name_under_the_body() {
        // A bird whose body is on the left, with a perch on the far right that hangs lower.
        let bird = block(30, 20, |x, y| (x < 10 && y < 10) || (x >= 27 && y < 18));
        let (joined, anchors) = bird.with_label(8, 4, 1);
        let (label_x, label_y, label_width) = anchors.label.expect("a label");
        assert_eq!(label_width, 8);
        assert!(label_x < 10, "under the body, not the perch: {label_x}");
        assert_eq!(
            label_y, 9,
            "just under the eroded body, above the perch's foot"
        );
        assert!(joined.get(label_x, label_y));
        assert_eq!(joined.runs().len(), joined.height);
    }

    #[test]
    fn wide_names_widen_the_footprint() {
        let bird = block(10, 10, |_, _| true);
        let (joined, anchors) = bird.with_label(30, 5, 2);
        assert_eq!(joined.width, 30);
        assert_eq!(
            anchors.art.0, 11,
            "centred on the body: 4.5 - 15 rounds to -11"
        );
        assert_eq!(joined.height, 10 + 2 + 5);
    }
}
