//! A colour with transparency.

use frame_core::Rgb;

/// An sRGB colour with straight (not premultiplied) alpha, 8 bits per channel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Opacity: 0 is transparent, 255 opaque.
    pub alpha: u8,
}

impl Rgba {
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    /// Creates a colour from its four channels.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates an opaque colour.
    #[must_use]
    pub const fn opaque(colour: Rgb) -> Self {
        Self::new(colour.red, colour.green, colour.blue, 255)
    }

    /// Returns the colour without its alpha.
    #[must_use]
    pub const fn rgb(self) -> Rgb {
        Rgb::new(self.red, self.green, self.blue)
    }
}
