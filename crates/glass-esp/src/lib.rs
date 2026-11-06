//! The Spectra 6 panels on the ESP32-S3 boards the firmware runs on.
//!
//! Each module is one board: its wiring of the panel, the order its panel is powered up and down,
//! and what its pins do through deep sleep. They hand the bus and pins to the panel drivers
//! (`panel-t133a01`, `panel-ed2208`), which do the talking.
//!
//! | Module | Board | Panel |
//! | --- | --- | --- |
//! | [`ee02`] | Seeed XIAO ePaper Display Board EE02 | 13.3", two T133A01 controllers |
//! | [`e1002`] | Seeed reTerminal E1002 | 7.3", one ED2208 controller |
//!
//! Built only with Espressif's Xtensa toolchain; the drivers' logic is tested on a laptop.

#![no_std]

pub mod e1002;
pub mod ee02;
mod refresh;

pub use refresh::Refresh;
