//! A picture's outline, kept small so it can be measured at any size.

use alloc::vec;
use alloc::vec::Vec;

use frame_core::Resolution;

/// Pixels above this alpha count as part of the bird, as in fugleramme.
const OPAQUE: u32 = 24;

/// The outline of a picture, for packing birds by their shapes rather than their boxes.
///
/// Holds a running sum of the picture's alpha, so the outline can be resampled to any size in
/// time proportional to that size. Pictures larger than [`Silhouette::MAX_SIDE`] are reduced
/// first; the packer only needs the outline to a pixel or two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Silhouette {
    width: usize,
    height: usize,
    /// Summed-area table of alpha, `(width + 1) * (height + 1)` entries.
    sums: Vec<u32>,
}

impl Silhouette {
    /// The longest side kept.
    pub const MAX_SIDE: usize = 480;

    /// Creates the outline of a picture of `resolution` from its alpha values, row by row.
    /// Returns `None` when the count does not match or the picture is empty.
    #[must_use]
    pub fn new(resolution: Resolution, alpha: &[u8]) -> Option<Self> {
        let Resolution { width, height } = resolution;
        if width == 0 || height == 0 || alpha.len() != width * height {
            return None;
        }
        let (reduced, width, height): (Vec<u8>, usize, usize) = reduce(alpha, width, height);
        let stride: usize = width + 1;
        let mut sums: Vec<u32> = vec![0; stride * (height + 1)];
        for y in 0..height {
            let mut row_sum: u32 = 0;
            for x in 0..width {
                row_sum += u32::from(reduced[y * width + x]);
                sums[(y + 1) * stride + x + 1] = sums[y * stride + x + 1] + row_sum;
            }
        }
        Some(Self {
            width,
            height,
            sums,
        })
    }

    /// Returns the picture's width divided by its height.
    #[must_use]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Returns the size of the picture drawn with its longest side `longest` pixels.
    pub(crate) fn scaled_size(&self, longest: usize) -> (usize, usize) {
        let longest: usize = longest.max(1);
        if self.width >= self.height {
            let height: usize = (self.height * longest + self.width / 2) / self.width;
            (longest, height.max(1))
        } else {
            let width: usize = (self.width * longest + self.height / 2) / self.height;
            (width.max(1), longest)
        }
    }

    /// Returns which pixels are part of the bird when drawn `width` by `height`, mirrored or
    /// not, row by row.
    pub(crate) fn mask(&self, width: usize, height: usize, mirrored: bool) -> Vec<bool> {
        let stride: usize = self.width + 1;
        let mut mask: Vec<bool> = Vec::with_capacity(width * height);
        for y in 0..height {
            let (top, bottom) = span(y, height, self.height);
            for x in 0..width {
                let column: usize = if mirrored { width - 1 - x } else { x };
                let (left, right) = span(column, width, self.width);
                let total: u32 = self.sums[bottom * stride + right]
                    + self.sums[top * stride + left]
                    - self.sums[top * stride + right]
                    - self.sums[bottom * stride + left];
                let area: u32 = u32::try_from((right - left) * (bottom - top)).unwrap_or(u32::MAX);
                mask.push(total > OPAQUE * area);
            }
        }
        mask
    }
}

/// The source pixels under target pixel `index` of `target`, at least one.
fn span(index: usize, target: usize, source: usize) -> (usize, usize) {
    let start: usize = (index * source / target).min(source - 1);
    let end: usize = ((index + 1) * source)
        .div_ceil(target)
        .clamp(start + 1, source);
    (start, end)
}

/// Shrinks `alpha` by a whole factor until its longest side is at most
/// [`Silhouette::MAX_SIDE`].
fn reduce(alpha: &[u8], width: usize, height: usize) -> (Vec<u8>, usize, usize) {
    let factor: usize = width.max(height).div_ceil(Silhouette::MAX_SIDE).max(1);
    if factor == 1 {
        return (alpha.to_vec(), width, height);
    }
    let (new_width, new_height) = (width.div_ceil(factor), height.div_ceil(factor));
    let mut reduced: Vec<u8> = Vec::with_capacity(new_width * new_height);
    for y in 0..new_height {
        for x in 0..new_width {
            let mut sum: u32 = 0;
            let mut count: u32 = 0;
            for source_y in y * factor..((y + 1) * factor).min(height) {
                for source_x in x * factor..((x + 1) * factor).min(width) {
                    sum += u32::from(alpha[source_y * width + source_x]);
                    count += 1;
                }
            }
            reduced.push(u8::try_from(sum / count.max(1)).unwrap_or(u8::MAX));
        }
    }
    (reduced, new_width, new_height)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resamples_and_mirrors() {
        // A 4x2 picture, opaque on its left half.
        let alpha: [u8; 8] = [255, 255, 0, 0, 255, 255, 0, 0];
        let outline = Silhouette::new(Resolution::new(4, 2), &alpha).expect("8 values");
        assert_eq!(outline.scaled_size(8), (8, 4));
        let mask: Vec<bool> = outline.mask(2, 1, false);
        assert_eq!(mask, [true, false]);
        assert_eq!(outline.mask(2, 1, true), [false, true]);
        let large: Vec<bool> = outline.mask(8, 4, false);
        assert_eq!(large.iter().filter(|&&set| set).count(), 16);
        assert!(Silhouette::new(Resolution::new(3, 3), &alpha).is_none());
    }

    #[test]
    fn large_pictures_are_reduced() {
        let alpha: Vec<u8> = vec![200; 1000 * 500];
        let outline = Silhouette::new(Resolution::new(1000, 500), &alpha).expect("sizes match");
        assert!(outline.width <= Silhouette::MAX_SIDE);
        assert!(libm::fabsf(outline.aspect_ratio() - 2.0) < 0.01);
    }
}
