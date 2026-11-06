//! The 7.3" panel on Seeed's reTerminal E1002: its pins and its SPI bus.
//!
//! The pins are those of Zephyr's board files for the E1002 (`reterminal_e1002_procpu.dts`,
//! Apache-2.0; see NOTICE.md): the panel on SPI2 with clock on GPIO7, data on GPIO9, chip
//! select GPIO10, data/command GPIO11, reset GPIO12 and BUSY GPIO13, at 4 MHz. The board has no
//! switch for the panel's power, so the controller's own deep sleep keeps it quiet between
//! refreshes. The microSD slot shares the bus with its own chip select (GPIO14) and power
//! (GPIO16), which this module leaves alone.

use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::peripherals::{GPIO7, GPIO9, GPIO10, GPIO11, GPIO12, GPIO13, SPI2};
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use panel_ed2208::{Ed2208, PanelError, Pins};

use crate::refresh::Refresh;

/// The E1002's wiring of the panel.
pub struct GlassPins<'d> {
    /// The SPI bus.
    pub spi: SPI2<'d>,
    /// SPI clock.
    pub sck: GPIO7<'d>,
    /// SPI data out.
    pub mosi: GPIO9<'d>,
    /// Chip select.
    pub cs: GPIO10<'d>,
    /// Data/command.
    pub dc: GPIO11<'d>,
    /// Reset.
    pub rst: GPIO12<'d>,
    /// BUSY from the controller, pulled up.
    pub busy: GPIO13<'d>,
}

/// Wakes the panel, shows `page` and puts the controller into deep sleep, then holds chip select
/// and reset high through the ESP32's deep sleep so neither floats. The controller goes to sleep
/// whatever happened.
pub fn show(pins: GlassPins<'_>, page: &[u8]) -> Refresh<PanelError> {
    let GlassPins {
        spi,
        sck,
        mosi,
        cs,
        dc,
        rst,
        busy,
    } = pins;
    let pins: Pins<Output<'_>, Input<'_>> = Pins {
        cs: released(Output::new(cs, Level::High, OutputConfig::default())),
        dc: released(Output::new(dc, Level::Low, OutputConfig::default())),
        rst: released(Output::new(rst, Level::High, OutputConfig::default())),
        busy: Input::new(busy, InputConfig::default().with_pull(Pull::Up)),
    };
    let bus: Spi<'_, Blocking> = new_bus(spi).with_sck(sck).with_mosi(mosi);
    let mut panel = Ed2208::new(bus, pins, Delay::new());
    let shown: Result<u32, PanelError> = panel.wake().and_then(|()| panel.show(page));
    let slept: Result<(), PanelError> = panel.sleep();
    let (bus, pins, _) = panel.release();
    drop(bus);
    for mut pin in [pins.cs, pins.rst] {
        pin.set_high();
        pin.set_pad_hold(true);
    }
    let mut dc: Output<'_> = pins.dc;
    dc.set_low();
    dc.set_pad_hold(true);
    Refresh { shown, slept }
}

/// The panel's bus: SPI mode 0 at 4 MHz, as Zephyr runs it on this board.
#[expect(
    clippy::expect_used,
    reason = "4 MHz in mode 0 is within what every ESP32-S3 SPI host supports"
)]
fn new_bus(spi: SPI2<'_>) -> Spi<'_, Blocking> {
    Spi::new(
        spi,
        Config::default()
            .with_frequency(Rate::from_mhz(4))
            .with_mode(Mode::_0),
    )
    .expect("a valid SPI configuration")
}

/// Lets go of the hold the last deep sleep left on `pin`, at the level it was created with.
fn released(mut pin: Output<'_>) -> Output<'_> {
    pin.set_pad_hold(false);
    pin
}
