//! Plain outlines, for tests of layout.

use alloc::vec::Vec;

use frame_core::Resolution;

use crate::art_image::ArtImage;
use crate::art_source::ArtSource;
use crate::rgba::Rgba;

/// The outline's colour.
const INK: Rgba = Rgba::new(40, 40, 40, 255);
/// The fill inside the outline.
const FILL: Rgba = Rgba::new(200, 200, 200, 255);

/// Gives every species the same plain oval with a dark outline: a cut-out of known size and
/// shape for tests of layout and rendering. It is not a bird; frames show fugleramme's plates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutlineArt {
    width: usize,
    height: usize,
}

impl OutlineArt {
    /// Creates a source whose ovals are `width_px` wide, three quarters of that high, and at
    /// least 4 by 3 pixels.
    #[must_use]
    pub fn new(width_px: usize) -> Self {
        let width: usize = width_px.max(4);
        Self {
            width,
            height: (width * 3 / 4).max(3),
        }
    }

    /// Returns the oval.
    ///
    /// # Panics
    ///
    /// Never: the pixels always match the size.
    #[must_use]
    #[expect(clippy::expect_used, reason = "one pixel is made per position")]
    pub fn outline(&self) -> ArtImage {
        let (width, height): (i64, i64) = (to_i64(self.width), to_i64(self.height));
        // Twice the coordinates, so the centre of a pixel is a whole number.
        let (rx, ry): (i64, i64) = (width, height);
        let ring: i64 = (width.min(height) / 12).max(1) * 2;
        let mut pixels: Vec<Rgba> = Vec::with_capacity(self.width * self.height);
        for y in 0..height {
            for x in 0..width {
                let (dx, dy): (i64, i64) = (2 * x + 1 - width, 2 * y + 1 - height);
                pixels.push(if !inside(dx, dy, rx, ry) {
                    Rgba::TRANSPARENT
                } else if inside(dx, dy, rx - ring, ry - ring) {
                    FILL
                } else {
                    INK
                });
            }
        }
        ArtImage::new(Resolution::new(self.width, self.height), pixels)
            .expect("one pixel per position")
    }
}

impl ArtSource for OutlineArt {
    fn art_for(&mut self, _scientific_name: &str) -> Option<ArtImage> {
        Some(self.outline())
    }
}

/// Whether (`dx`, `dy`) lies inside the ellipse with radii `rx` and `ry`.
fn inside(dx: i64, dy: i64, rx: i64, ry: i64) -> bool {
    rx > 0 && ry > 0 && dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry
}

fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_a_cut_out_oval_of_the_asked_width() {
        let picture: ArtImage = OutlineArt::new(80).outline();
        assert_eq!(picture.resolution(), Resolution::new(80, 60));
        assert_eq!(picture.get(0, 0).alpha, 0);
        assert_eq!(picture.get(40, 30), FILL);
        assert_eq!(picture.get(0, 30), INK);
    }
}
