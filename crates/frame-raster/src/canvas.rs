//! A full-colour drawing surface.

use alloc::vec;
use alloc::vec::Vec;

use frame_core::{Resolution, Rgb};

/// A full-colour page, row by row, before dithering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    resolution: Resolution,
    pixels: Vec<Rgb>,
}

impl Canvas {
    /// Creates a canvas of `resolution` filled with `background`.
    #[must_use]
    pub fn new(resolution: Resolution, background: Rgb) -> Self {
        Self {
            resolution,
            pixels: vec![background; resolution.area()],
        }
    }

    /// Returns the canvas size.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Returns all pixels, row by row.
    #[must_use]
    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    /// Blends `colour` over the pixel at (`x`, `y`) with `coverage` from 0 (none) to 255 (full).
    /// Pixels outside the canvas are ignored.
    pub fn blend(&mut self, x: i32, y: i32, colour: Rgb, coverage: u8) {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return;
        };
        if x >= self.resolution.width || y >= self.resolution.height || coverage == 0 {
            return;
        }
        let Some(pixel) = self.pixels.get_mut(y * self.resolution.width + x) else {
            return;
        };
        let alpha: u16 = u16::from(coverage);
        let mix = |under: u8, over: u8| -> u8 {
            let mixed: u16 =
                (u16::from(under) * (255 - alpha) + u16::from(over) * alpha + 127) / 255;
            u8::try_from(mixed).unwrap_or(u8::MAX)
        };
        *pixel = Rgb::new(
            mix(pixel.red, colour.red),
            mix(pixel.green, colour.green),
            mix(pixel.blue, colour.blue),
        );
    }
}
