//! A page as a grid of inks.

use alloc::vec;
use alloc::vec::Vec;

use crate::ink::Ink;
use crate::resolution::Resolution;
use crate::rotation::Rotation;

/// A page where every pixel is one of the six inks, stored row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedImage {
    resolution: Resolution,
    pixels: Vec<Ink>,
}

impl IndexedImage {
    /// Creates an image of `resolution` with every pixel set to `fill`.
    #[must_use]
    pub fn new(resolution: Resolution, fill: Ink) -> Self {
        Self {
            resolution,
            pixels: vec![fill; resolution.area()],
        }
    }

    /// Creates an image from `pixels`, row by row, or returns `None` when their count does not
    /// match `resolution`.
    #[must_use]
    pub fn from_pixels(resolution: Resolution, pixels: Vec<Ink>) -> Option<Self> {
        (pixels.len() == resolution.area()).then_some(Self { resolution, pixels })
    }

    /// Returns the image's size.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Returns all pixels, row by row.
    #[must_use]
    pub fn pixels(&self) -> &[Ink] {
        &self.pixels
    }

    /// Returns the ink at (`x`, `y`), or `None` outside the image.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<Ink> {
        if x >= self.resolution.width || y >= self.resolution.height {
            return None;
        }
        self.pixels.get(y * self.resolution.width + x).copied()
    }

    /// Sets the ink at (`x`, `y`). Pixels outside the image are ignored, so callers can draw
    /// shapes that run off the edge.
    pub fn set(&mut self, x: usize, y: usize, ink: Ink) {
        if x >= self.resolution.width || y >= self.resolution.height {
            return;
        }
        let index = y * self.resolution.width + x;
        if let Some(pixel) = self.pixels.get_mut(index) {
            *pixel = ink;
        }
    }

    /// Returns the image turned counter-clockwise by `rotation`, the way PIL's
    /// `Image.transpose(ROTATE_90)` and friends turn it.
    #[must_use]
    pub fn rotated(&self, rotation: Rotation) -> Self {
        let Resolution { width, height } = self.resolution;
        let quarter_turns: u8 = rotation.quarter_turns();
        let resolution: Resolution = if rotation.is_portrait() {
            self.resolution.transposed()
        } else {
            self.resolution
        };
        let mut pixels: Vec<Ink> = Vec::with_capacity(resolution.area());
        for new_y in 0..resolution.height {
            for new_x in 0..resolution.width {
                let (old_x, old_y) = match quarter_turns {
                    0 => (new_x, new_y),
                    1 => (width - 1 - new_y, new_x),
                    2 => (width - 1 - new_x, height - 1 - new_y),
                    _ => (new_y, height - 1 - new_x),
                };
                pixels.push(self.pixels[old_y * width + old_x]);
            }
        }
        Self { resolution, pixels }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn numbered() -> IndexedImage {
        // 3x2:  Black White Yellow
        //       Red   Blue  Green
        IndexedImage::from_pixels(Resolution::new(3, 2), Ink::ALL.to_vec())
            .expect("six pixels fill a 3x2 image")
    }

    #[test]
    fn quarter_turn_counter_clockwise_moves_the_top_right_corner_to_the_top_left() {
        let turned = numbered().rotated(Rotation::Degrees90);
        assert_eq!(turned.resolution(), Resolution::new(2, 3));
        assert_eq!(
            turned.pixels(),
            &[
                Ink::Yellow,
                Ink::Green,
                Ink::White,
                Ink::Blue,
                Ink::Black,
                Ink::Red
            ]
        );
    }

    #[test]
    fn four_quarter_turns_are_the_identity() {
        let image = numbered();
        let mut turned = image.clone();
        for _ in 0..4 {
            turned = turned.rotated(Rotation::Degrees90);
        }
        assert_eq!(turned, image);
    }

    #[test]
    fn three_quarters_equal_one_quarter_the_other_way() {
        let image = numbered();
        let once = image.rotated(Rotation::Degrees270);
        let thrice = image
            .rotated(Rotation::Degrees90)
            .rotated(Rotation::Degrees90)
            .rotated(Rotation::Degrees90);
        assert_eq!(once, thrice);
    }

    #[test]
    fn set_outside_is_ignored() {
        let mut image = numbered();
        image.set(3, 0, Ink::Black);
        image.set(0, 2, Ink::Black);
        assert_eq!(image, numbered());
    }
}
