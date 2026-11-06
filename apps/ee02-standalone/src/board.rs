//! The board the firmware is built for, picked by a Cargo feature: its panel and its buttons.
//!
//! | | XIAO EE02 (`board-ee02`) | reTerminal E1002 (`board-e1002`) |
//! | --- | --- | --- |
//! | Panel | 13.3", `glass_esp::ee02` | 7.3", `glass_esp::e1002` |
//! | Refresh, previous, next | GPIO2, GPIO3, GPIO5 | GPIO3, GPIO5, GPIO4 |
//!
//! The pins are `ee02-thin`'s; see its `board.rs` for their sources.

#[cfg(all(feature = "board-ee02", feature = "board-e1002"))]
compile_error!("pick one board: build board-e1002 with --no-default-features");
#[cfg(not(any(feature = "board-ee02", feature = "board-e1002")))]
compile_error!("pick a board: the board-ee02 or the board-e1002 feature");

use frame_core::PanelSize;

/// The board's name, for the log.
#[cfg(feature = "board-ee02")]
pub const NAME: &str = "XIAO EE02 and 13.3\" panel";
/// The board's name, for the log.
#[cfg(feature = "board-e1002")]
pub const NAME: &str = "reTerminal E1002";

/// The board's panel.
#[cfg(feature = "board-ee02")]
pub const PANEL: PanelSize = PanelSize::Inches13Point3;
/// The board's panel.
#[cfg(feature = "board-e1002")]
pub const PANEL: PanelSize = PanelSize::Inches7Point3;

/// The board's buttons, taken from `peripherals`. Macros, because the firmware moves the
/// peripherals out of the struct one by one.
#[cfg(feature = "board-ee02")]
macro_rules! buttons {
    ($peripherals:ident) => {
        crate::buttons::Buttons::new($peripherals.GPIO2, $peripherals.GPIO3, $peripherals.GPIO5)
    };
}
/// The board's buttons, taken from `peripherals`.
#[cfg(feature = "board-e1002")]
macro_rules! buttons {
    ($peripherals:ident) => {
        crate::buttons::Buttons::new($peripherals.GPIO3, $peripherals.GPIO5, $peripherals.GPIO4)
    };
}

/// Shows a page on the EE02's panel, taking its pins from `peripherals`. A macro, because the
/// firmware has already moved other peripherals out of the struct.
#[cfg(feature = "board-ee02")]
macro_rules! show {
    ($peripherals:ident, $page:expr) => {
        glass_esp::ee02::show(
            glass_esp::ee02::GlassPins {
                spi: $peripherals.SPI2,
                sck: $peripherals.GPIO7,
                mosi: $peripherals.GPIO9,
                cs0: $peripherals.GPIO44,
                cs1: $peripherals.GPIO41,
                dc: $peripherals.GPIO10,
                rst: $peripherals.GPIO38,
                busy: $peripherals.GPIO4,
                power: $peripherals.GPIO43,
            },
            $page,
        )
    };
}

/// Shows a page on the E1002's panel, taking its pins from `peripherals`. A macro, because the
/// firmware has already moved other peripherals out of the struct.
#[cfg(feature = "board-e1002")]
macro_rules! show {
    ($peripherals:ident, $page:expr) => {
        glass_esp::e1002::show(
            glass_esp::e1002::GlassPins {
                spi: $peripherals.SPI2,
                sck: $peripherals.GPIO7,
                mosi: $peripherals.GPIO9,
                cs: $peripherals.GPIO10,
                dc: $peripherals.GPIO11,
                rst: $peripherals.GPIO12,
                busy: $peripherals.GPIO13,
            },
            $page,
        )
    };
}

pub(crate) use buttons;
pub(crate) use show;
