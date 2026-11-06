//! The panel on the EE02: its power switch, its pins and its SPI bus.
//!
//! The wiring and the power-up and power-down order follow fugleramme's EE02 firmware
//! (`examples/ee02/main.py`, MIT, Copyright (c) 2026 Arne Giacomo Munthe-Kaas; see NOTICE.md).

use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO4, GPIO7, GPIO9, GPIO10, GPIO38, GPIO41, GPIO43, GPIO44, SPI2};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use panel_t133a01::{PanelError, Pins, T133a01};

/// The EE02's wiring of the panel, from Seeed's `XIAO_ePaper_Board_Configs.h` by way of
/// fugleramme's EE02 firmware.
pub struct GlassPins<'d> {
    /// The SPI bus.
    pub spi: SPI2<'d>,
    /// SPI clock (XIAO D8).
    pub sck: GPIO7<'d>,
    /// SPI data out (XIAO D10).
    pub mosi: GPIO9<'d>,
    /// Chip select of the first controller.
    pub cs0: GPIO44<'d>,
    /// Chip select of the second controller.
    pub cs1: GPIO41<'d>,
    /// Data/command.
    pub dc: GPIO10<'d>,
    /// Reset.
    pub rst: GPIO38<'d>,
    /// BUSY from the controllers.
    pub busy: GPIO4<'d>,
    /// The panel's load switch, high for power.
    pub power: GPIO43<'d>,
}

/// Powers the panel, shows `page`, powers it off and holds its pins low through deep sleep, so
/// the idle panel is not fed through them. Returns how many milliseconds the refresh took.
///
/// # Errors
///
/// Returns [`PanelError`] when the panel fails to take or show the page; it is powered off and its
/// pins held either way.
pub fn show(pins: GlassPins<'_>, page: &[u8]) -> Result<u32, PanelError> {
    let GlassPins {
        spi,
        mut sck,
        mut mosi,
        cs0,
        cs1,
        dc,
        rst,
        busy,
        power,
    } = pins;
    // Driven high at once: a short cut of an energised panel has broken refreshes.
    let mut power: Output<'_> = Output::new(power, Level::High, OutputConfig::default());
    let pins: Pins<Output<'_>, Input<'_>> = Pins {
        cs0: released(Output::new(cs0, Level::High, OutputConfig::default())),
        cs1: released(Output::new(cs1, Level::High, OutputConfig::default())),
        dc: released(Output::new(dc, Level::Low, OutputConfig::default())),
        rst: released(Output::new(rst, Level::High, OutputConfig::default())),
        busy: Input::new(busy, InputConfig::default()),
    };
    // The bus's pins were held low too; let go before the bus takes them.
    Output::new(sck.reborrow(), Level::Low, OutputConfig::default()).set_pad_hold(false);
    Output::new(mosi.reborrow(), Level::Low, OutputConfig::default()).set_pad_hold(false);
    let bus: Spi<'_, Blocking> = new_bus(spi)
        .with_sck(sck.reborrow())
        .with_mosi(mosi.reborrow());
    let mut panel = T133a01::new(bus, pins, Delay::new());
    let shown: Result<u32, PanelError> = panel.wake().and_then(|()| panel.show(page));
    // The picture is on the glass by now, so a failed power-off only gets a log line: the power
    // is cut below either way.
    if let Err(error) = panel.sleep() {
        esp_println::println!("ee02: powering the panel off: {error}");
    }
    let (bus, pins, _) = panel.release();
    power.set_low();
    drop(bus);
    for mut pin in [pins.cs0, pins.cs1, pins.dc, pins.rst] {
        pin.set_low();
        pin.set_pad_hold(true);
    }
    for mut pin in [
        Output::new(sck, Level::Low, OutputConfig::default()),
        Output::new(mosi, Level::Low, OutputConfig::default()),
    ] {
        pin.set_pad_hold(true);
    }
    shown
}

/// The panel's bus: SPI mode 0 at 10 MHz.
#[expect(
    clippy::expect_used,
    reason = "10 MHz in mode 0 is within what every ESP32-S3 SPI host supports"
)]
fn new_bus(spi: SPI2<'_>) -> Spi<'_, Blocking> {
    Spi::new(
        spi,
        Config::default()
            .with_frequency(Rate::from_mhz(10))
            .with_mode(Mode::_0),
    )
    .expect("a valid SPI configuration")
}

/// Lets go of the hold the last deep sleep left on `pin`, at the level it was created with.
fn released(mut pin: Output<'_>) -> Output<'_> {
    pin.set_pad_hold(false);
    pin
}
