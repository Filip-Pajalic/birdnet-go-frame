//! An sRGB colour with 8 bits per channel.

/// An sRGB colour with 8 bits per channel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
}

impl Rgb {
    /// Pure white, the colour of an empty page before dithering.
    pub const WHITE: Self = Self::new(255, 255, 255);

    /// Pure black.
    pub const BLACK: Self = Self::new(0, 0, 0);

    /// Creates a colour from its three channels.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}
