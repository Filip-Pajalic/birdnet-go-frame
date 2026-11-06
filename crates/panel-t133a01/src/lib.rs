//! A driver for the 13.3" Spectra 6 e-ink panel and its two T133A01 controllers.
//!
//! The panel is 1200x1600 in the controllers' own orientation. Each controller drives one half
//! of every row, and both share one SPI bus with a chip select each. The driver takes the bytes
//! `/panel` serves (`frame_core::panel_bytes::pack`): every row's left half for the first
//! controller, then every row's right half for the second.
//!
//! The command sequence follows fugleramme's EE02 firmware (MIT, Copyright (c) 2026 Arne Giacomo
//! Munthe-Kaas; see NOTICE.md), itself inspired by Seeed's `Driver_T133A01.cpp` (MIT). It runs
//! on any `embedded-hal` 1.0 implementation, so the same code drives the panel from an ESP32 and
//! from a test's mock bus.
//!
//! ```text
//! wake()  reset, init, PLL, power on
//! draw()  send each controller its half
//! refresh()  run the refresh and follow BUSY for its ~30 s
//! sleep()  power off the controllers
//! ```

#![no_std]

mod commands;
mod panel_error;
mod t133a01;

pub use panel_error::PanelError;
pub use t133a01::{PAGE_LEN, Pins, T133a01};
