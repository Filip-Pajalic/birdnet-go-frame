//! A driver for the 7.3" Spectra 6 e-ink panel (800x480) and its ED2208 controller.
//!
//! This is the panel on Seeed's reTerminal E1002 (sold as EL073TF1), Waveshare's 7.3" e-Paper
//! (E) and Good Display's GDEP073E01. One controller drives the whole panel, on an SPI bus with
//! one chip select. The driver takes the bytes `/panel?panel=7.3` serves
//! (`frame_core::panel_bytes::pack`): the panel's native landscape rows, two pixels a byte.
//!
//! The register values follow Zephyr's `display_ed2208_gca.c` driver (Apache-2.0, Copyright (c)
//! 2026 The Zephyr Project Contributors; see NOTICE.md). It runs on any `embedded-hal` 1.0
//! implementation, so the same code drives the panel from an ESP32, from a Raspberry Pi and from
//! a test's mock bus.
//!
//! ```text
//! wake()     reset, init, power on
//! draw()     send the page
//! refresh()  run the refresh and follow BUSY for its ~20 s
//! sleep()    power off, then deep sleep (only a reset wakes it)
//! ```

#![no_std]

mod commands;
mod ed2208;
mod panel_error;

pub use ed2208::{Ed2208, PAGE_LEN, Pins};
pub use panel_error::PanelError;
