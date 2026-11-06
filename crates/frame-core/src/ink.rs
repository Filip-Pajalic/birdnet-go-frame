//! The six inks of an E Ink Spectra 6 panel.

use crate::rgb::Rgb;

/// One of the six inks a Spectra 6 panel can show.
///
/// The declaration order is the palette order used by fugleramme and Pimoroni's driver (black,
/// white, yellow, red, blue, green), which [`Ink::palette_index`] exposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ink {
    /// Black.
    Black,
    /// White (the paper).
    White,
    /// Yellow.
    Yellow,
    /// Red.
    Red,
    /// Blue.
    Blue,
    /// Green.
    Green,
}

impl Ink {
    /// Every ink, in palette order.
    pub const ALL: [Self; 6] = [
        Self::Black,
        Self::White,
        Self::Yellow,
        Self::Red,
        Self::Blue,
        Self::Green,
    ];

    /// Returns the ink's position in the palette, from 0 (black) to 5 (green).
    #[must_use]
    pub fn palette_index(self) -> u8 {
        self as u8
    }

    /// Returns the ink at `index` in the palette, or `None` past the sixth.
    #[must_use]
    pub fn from_palette_index(index: u8) -> Option<Self> {
        Self::ALL.get(usize::from(index)).copied()
    }

    /// Returns the 4-bit code the T133A01 controllers use for this ink. The controllers skip
    /// code 4.
    #[must_use]
    pub fn controller_code(self) -> u8 {
        match self {
            Self::Black => 0,
            Self::White => 1,
            Self::Yellow => 2,
            Self::Red => 3,
            Self::Blue => 5,
            Self::Green => 6,
        }
    }

    /// Returns the ink a controller code stands for, or `None` for an unused code.
    #[must_use]
    pub fn from_controller_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Black),
            1 => Some(Self::White),
            2 => Some(Self::Yellow),
            3 => Some(Self::Red),
            5 => Some(Self::Blue),
            6 => Some(Self::Green),
            _ => None,
        }
    }

    /// Returns the colour to dither against for this ink.
    ///
    /// These are fugleramme's blend of the driver's saturated and desaturated palettes at 0.5,
    /// which measures colour distance roughly as the panel shows it.
    #[must_use]
    pub fn dither_colour(self) -> Rgb {
        match self {
            Self::Black => Rgb::new(0, 0, 0),
            Self::White => Rgb::new(208, 209, 210),
            Self::Yellow => Rgb::new(231, 222, 35),
            Self::Red => Rgb::new(205, 36, 37),
            Self::Blue => Rgb::new(30, 29, 174),
            Self::Green => Rgb::new(29, 173, 35),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn controller_codes_round_trip() {
        for ink in Ink::ALL {
            assert_eq!(Ink::from_controller_code(ink.controller_code()), Some(ink));
        }
        assert_eq!(Ink::from_controller_code(4), None);
    }

    #[test]
    fn palette_indices_round_trip() {
        for ink in Ink::ALL {
            assert_eq!(Ink::from_palette_index(ink.palette_index()), Some(ink));
        }
        assert_eq!(Ink::from_palette_index(6), None);
    }
}
