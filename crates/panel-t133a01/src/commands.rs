//! The controllers' commands and the start-up sequence.

/// Which controllers a command goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Chips {
    /// The first controller (CS0): every row's left half.
    Primary,
    /// The second controller (CS1): every row's right half.
    Secondary,
    /// Both at once.
    Both,
}

impl Chips {
    pub(crate) fn primary(self) -> bool {
        matches!(self, Self::Primary | Self::Both)
    }

    pub(crate) fn secondary(self) -> bool {
        matches!(self, Self::Secondary | Self::Both)
    }
}

/// Power off.
pub(crate) const POWER_OFF: u8 = 0x02;
/// Power on.
pub(crate) const POWER_ON: u8 = 0x04;
/// Data start transmission: pixel data follows.
pub(crate) const DATA: u8 = 0x10;
/// Display refresh.
pub(crate) const REFRESH: u8 = 0x12;
/// Phase-locked loop: the frame rate.
pub(crate) const PLL: u8 = 0x30;
/// Cascade setting, sent before each controller's data.
pub(crate) const CASCADE: u8 = 0xe0;

/// Sent after reset, in order, before the PLL and power on.
pub(crate) const INIT: [(Chips, u8, &[u8]); 16] = [
    (
        Chips::Primary,
        0x74,
        &[0x00, 0x0c, 0x0c, 0xd9, 0xdd, 0xdd, 0x15, 0x15, 0x55],
    ),
    (Chips::Both, 0xf0, &[0x49, 0x55, 0x13, 0x5d, 0x05, 0x10]),
    (Chips::Both, 0x00, &[0xdf, 0x69]),
    (Chips::Primary, 0xa5, &[0x44, 0x54, 0x00]),
    (Chips::Both, 0x50, &[0x37]),
    (Chips::Both, 0x60, &[0x03, 0x03]),
    (Chips::Both, 0x86, &[0x10]),
    (Chips::Both, 0xe3, &[0x22]),
    // Resolution per controller: 1200 (0x04b0) by 800 (0x0320), in the controllers' terms.
    (Chips::Both, 0x61, &[0x04, 0xb0, 0x03, 0x20]),
    (Chips::Primary, 0x01, &[0x0f, 0x00, 0x28, 0x2c, 0x28, 0x38]),
    (Chips::Primary, 0xb6, &[0x07]),
    (Chips::Primary, 0x06, &[0xe0, 0x20]),
    (Chips::Primary, 0xb7, &[0x01]),
    (Chips::Primary, 0x05, &[0xe0, 0x20]),
    (Chips::Primary, 0xb0, &[0x01]),
    (Chips::Primary, 0xb1, &[0x02]),
];
