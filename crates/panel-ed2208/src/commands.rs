//! The controller's commands and its start-up sequence.

/// Power off.
pub(crate) const POWER_OFF: u8 = 0x02;
/// Power on.
pub(crate) const POWER_ON: u8 = 0x04;
/// Deep sleep; takes [`DEEP_SLEEP_CHECK`].
pub(crate) const DEEP_SLEEP: u8 = 0x07;
/// The byte deep sleep must be sent with, so a stray command cannot trigger it.
pub(crate) const DEEP_SLEEP_CHECK: u8 = 0xa5;
/// Data start transmission: pixel data follows.
pub(crate) const DATA: u8 = 0x10;
/// Display refresh.
pub(crate) const REFRESH: u8 = 0x12;

/// Sent after reset, in order, before [`POWER_ON`]: command, then its data.
pub(crate) const INIT: [(u8, &[u8]); 19] = [
    // Command header: unlocks the settings that follow.
    (0xaa, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18]),
    // Power setting.
    (0x01, &[0x3f, 0x00, 0x32, 0x2a, 0x0e, 0x2a]),
    // Panel setting.
    (0x00, &[0x5f, 0x69]),
    // Power off sequence.
    (0x03, &[0x00, 0x54, 0x00, 0x44]),
    // Booster soft start, three phases.
    (0x05, &[0x40, 0x1f, 0x1f, 0x2c]),
    (0x06, &[0x6f, 0x1f, 0x16, 0x25]),
    (0x08, &[0x6f, 0x1f, 0x1f, 0x22]),
    (0x13, &[0x00, 0x04]),
    // Phase-locked loop: the frame rate.
    (0x30, &[0x02]),
    // Temperature sensor: the built-in one.
    (0x41, &[0x00]),
    // VCOM and data interval.
    (0x50, &[0x3f]),
    // Gate and source non-overlap.
    (0x60, &[0x02, 0x00]),
    // Resolution: 800 (0x0320) by 480 (0x01e0).
    (0x61, &[0x03, 0x20, 0x01, 0xe0]),
    (0x82, &[0x1e]),
    (0x84, &[0x01]),
    (0x86, &[0x00]),
    // Power saving.
    (0xe3, &[0x2f]),
    (0xe0, &[0x00]),
    (0xe6, &[0x00]),
];
