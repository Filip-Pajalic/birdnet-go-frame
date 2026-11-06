//! Packing footprints outwards from the middle of the page.
//!
//! Ported from fugleramme's `render/packing.py` (MIT, © 2026 Arne Giacomo Munthe-Kaas).

use alloc::vec;
use alloc::vec::Vec;
use core::f32::consts::PI;

use crate::footprint::Footprint;

/// Pixels between candidate positions, along each ring and between rings.
const STEP: f32 = 6.0;
/// Rows tested before a whole footprint, one from each horizontal band.
const PROBE_BANDS: usize = 3;

/// One footprint, ready to test against the page.
struct Shape {
    width: usize,
    height: usize,
    runs: Vec<Vec<(usize, usize)>>,
    probes: Vec<usize>,
}

impl Shape {
    fn new(footprint: &Footprint) -> Self {
        let runs: Vec<Vec<(usize, usize)>> = footprint.runs();
        // The densest row of each band: most positions on a filling page collide, and one row
        // costs a fraction of the whole. Banded so the probes do not all land in the body and
        // catch the same collisions.
        let band: usize = footprint.height.div_ceil(PROBE_BANDS).max(1);
        let probes: Vec<usize> = (0..footprint.height)
            .step_by(band)
            .filter_map(|start| {
                (start..(start + band).min(footprint.height))
                    .map(|y| (runs[y].iter().map(|(a, b)| b - a).sum::<usize>(), y))
                    .filter(|(filled, _)| *filled > 0)
                    .max()
                    .map(|(_, y)| y)
            })
            .collect();
        Self {
            width: footprint.width,
            height: footprint.height,
            runs,
            probes,
        }
    }
}

/// Which pixels of the page are taken, one bit each.
struct Page {
    width: usize,
    height: usize,
    words: usize,
    bits: Vec<u64>,
}

impl Page {
    fn new(width: usize, height: usize) -> Self {
        let words: usize = width.div_ceil(64);
        Self {
            width,
            height,
            words,
            bits: vec![0; words * height],
        }
    }

    /// Whether any pixel from `start` to `end` (exclusive) on row `y` is taken.
    fn taken(&self, y: usize, start: usize, end: usize) -> bool {
        let row: &[u64] = &self.bits[y * self.words..(y + 1) * self.words];
        let (first, last) = (start / 64, (end - 1) / 64);
        (first..=last).any(|word| {
            let low: usize = if word == first { start % 64 } else { 0 };
            let high: usize = if word == last { (end - 1) % 64 } else { 63 };
            let mask: u64 = (u64::MAX >> (63 - high)) & (u64::MAX << low);
            row[word] & mask != 0
        })
    }

    fn take(&mut self, y: usize, start: usize, end: usize) {
        for x in start..end {
            self.bits[y * self.words + x / 64] |= 1 << (x % 64);
        }
    }

    fn row_fits(&self, shape: &Shape, row: usize, x: usize, y: usize) -> bool {
        shape.runs[row]
            .iter()
            .all(|&(start, end)| !self.taken(y + row, x + start, x + end))
    }

    fn fits(&self, shape: &Shape, x: usize, y: usize) -> bool {
        // A colliding probe is a real collision, so probing first only ever skips the full
        // test for a position it would have failed anyway.
        shape
            .probes
            .iter()
            .all(|&row| self.row_fits(shape, row, x, y))
            && (0..shape.height).all(|row| self.row_fits(shape, row, x, y))
    }

    fn place(&mut self, shape: &Shape, x: usize, y: usize) {
        for (row, runs) in shape.runs.iter().enumerate() {
            for &(start, end) in runs {
                self.take(y + row, x + start, x + end);
            }
        }
    }

    /// The first position on an outward walk from the middle where `shape` fits.
    fn first_free(&self, shape: &Shape) -> Option<(usize, usize)> {
        if shape.width > self.width || shape.height > self.height {
            return None;
        }
        let (centre_x, centre_y) = (self.width as f32 / 2.0, self.height as f32 / 2.0);
        let reach: f32 = libm::hypotf(self.width as f32, self.height as f32);
        let half: (f32, f32) = (shape.width as f32 / 2.0, shape.height as f32 / 2.0);
        let try_at = |px: f32, py: f32| -> Option<(usize, usize)> {
            let (left, top) = (px - half.0, py - half.1);
            if left < 0.0 || top < 0.0 {
                return None;
            }
            let (x, y) = (left as usize, top as usize);
            let inside: bool = x + shape.width <= self.width && y + shape.height <= self.height;
            (inside && self.fits(shape, x, y)).then_some((x, y))
        };
        if let Some(spot) = try_at(centre_x, centre_y) {
            return Some(spot);
        }
        let mut radius: f32 = STEP;
        while radius <= reach {
            let count: usize = ((2.0 * PI * radius / STEP) as usize).max(8);
            for index in 0..count {
                let angle: f32 = 2.0 * PI * index as f32 / count as f32;
                let spot = try_at(
                    centre_x + radius * libm::cosf(angle),
                    centre_y + radius * libm::sinf(angle),
                );
                if spot.is_some() {
                    return spot;
                }
            }
            radius += STEP;
        }
        None
    }
}

/// Places every footprint, largest first as given, with no overlap and wholly inside a
/// `width` by `height` area, each as near the middle as it fits. Returns the top-left corners,
/// or `None` when one does not fit.
pub(crate) fn pack(
    footprints: &[Footprint],
    width: usize,
    height: usize,
) -> Option<Vec<(usize, usize)>> {
    let mut page = Page::new(width, height);
    let mut corners: Vec<(usize, usize)> = Vec::with_capacity(footprints.len());
    for footprint in footprints {
        let shape = Shape::new(footprint);
        let (x, y) = page.first_free(&shape)?;
        page.place(&shape, x, y);
        corners.push((x, y));
    }
    Some(corners)
}

#[cfg(test)]
mod test {
    use super::*;

    fn square(side: usize) -> Footprint {
        Footprint::of_art(side, side, vec![true; side * side])
    }

    #[test]
    fn biggest_goes_in_the_middle_and_none_overlap() {
        let footprints: Vec<Footprint> = vec![square(40), square(20), square(20), square(20)];
        let corners = pack(&footprints, 120, 100).expect("room enough");
        assert_eq!(corners[0], (40, 30));
        let mut page = Page::new(120, 100);
        for (footprint, &(x, y)) in footprints.iter().zip(&corners) {
            let shape = Shape::new(footprint);
            assert!(page.fits(&shape, x, y), "overlap at {x},{y}");
            page.place(&shape, x, y);
        }
    }

    #[test]
    fn reports_when_nothing_fits() {
        assert!(pack(&[square(50), square(50)], 60, 60).is_none());
        assert!(pack(&[square(70)], 60, 60).is_none());
    }

    #[test]
    fn bit_ranges() {
        let mut page = Page::new(200, 1);
        page.take(0, 63, 66);
        assert!(page.taken(0, 0, 64));
        assert!(page.taken(0, 65, 200));
        assert!(!page.taken(0, 0, 63));
        assert!(!page.taken(0, 66, 200));
    }
}
