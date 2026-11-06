//! The panel on the Pi's SPI bus and GPIO lines.

use frame_core::PanelSize;

use crate::config::Wiring;

/// The panel and its power switch, opened from the wiring in the settings.
#[cfg(target_os = "linux")]
pub struct Board {
    glass: linux::Glassware,
    power: Option<linux_embedded_hal::CdevPin>,
}

/// Stands in for the panel where there is no spidev: opening it always fails.
#[cfg(not(target_os = "linux"))]
pub struct Board;

#[cfg(target_os = "linux")]
impl Board {
    /// Opens the SPI device and GPIO lines `wiring` names for `panel`.
    ///
    /// # Errors
    ///
    /// Returns what went wrong, naming the device or line, when one cannot be opened.
    pub fn open(wiring: &Wiring, panel: PanelSize) -> Result<Self, String> {
        let (glass, power) = linux::open(wiring, panel)?;
        Ok(Self { glass, power })
    }

    /// Powers the panel, shows `page` on it, and powers it off again. Returns how many
    /// milliseconds the refresh took.
    ///
    /// # Errors
    ///
    /// Returns the driver's error when the panel fails to take or show the page; the panel is
    /// put to sleep and its power cut either way.
    pub fn show(&mut self, page: &[u8]) -> Result<u32, String> {
        use embedded_hal::digital::OutputPin;

        if let Some(power) = &mut self.power {
            power
                .set_high()
                .map_err(|error| format!("panel power: {error}"))?;
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let shown: Result<u32, String> = self.glass.show(page);
        if let Some(power) = &mut self.power {
            if let Err(error) = power.set_low() {
                eprintln!("linux-panel: cutting the panel's power: {error}");
            }
        }
        shown
    }
}

#[cfg(not(target_os = "linux"))]
impl Board {
    /// Fails: the panel can be driven only on Linux.
    ///
    /// # Errors
    ///
    /// Always, saying to write pages to a file with `PANEL_OUT` instead.
    pub fn open(_wiring: &Wiring, _panel: PanelSize) -> Result<Self, String> {
        Err(String::from(
            "the panel can be driven only on Linux; set PANEL_OUT to write pages to a file",
        ))
    }

    /// Never called: [`Board::open`] never gives a board.
    ///
    /// # Errors
    ///
    /// Always.
    pub fn show(&mut self, _page: &[u8]) -> Result<u32, String> {
        Err(String::from("no panel"))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use embedded_hal::digital::PinState;
    use frame_core::{Glass, PanelSize};
    use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
    use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
    use linux_embedded_hal::{CdevPin, Delay, SpidevBus};
    use panel_ed2208::Ed2208;
    use panel_t133a01::T133a01;

    use crate::chunked_bus::{self, ChunkedBus};
    use crate::config::Wiring;

    /// The bus the drivers get.
    type Bus = ChunkedBus<SpidevBus>;

    /// Either panel's driver.
    pub enum Glassware {
        /// The 13.3" panel.
        Large(T133a01<Bus, CdevPin, CdevPin, Delay>),
        /// The 7.3" panel.
        Small(Ed2208<Bus, CdevPin, CdevPin, Delay>),
    }

    impl Glassware {
        /// Wakes the panel, shows `page` and puts the panel to sleep.
        pub fn show(&mut self, page: &[u8]) -> Result<u32, String> {
            match self {
                Self::Large(glass) => show_on(glass, page),
                Self::Small(glass) => show_on(glass, page),
            }
        }
    }

    /// Wakes `glass`, shows `page` and puts it to sleep, even when showing failed.
    fn show_on<G: Glass>(glass: &mut G, page: &[u8]) -> Result<u32, String> {
        glass.wake().map_err(|error| error.to_string())?;
        let shown: Result<u32, String> = glass.show(page).map_err(|error| error.to_string());
        if let Err(error) = glass.sleep() {
            eprintln!("linux-panel: putting the panel to sleep: {error}");
        }
        shown
    }

    /// Opens the bus, the panel's lines and its power switch.
    pub fn open(wiring: &Wiring, panel: PanelSize) -> Result<(Glassware, Option<CdevPin>), String> {
        let mut spi: SpidevBus = SpidevBus::open(&wiring.spi_device)
            .map_err(|error| format!("{}: {error}", wiring.spi_device.display()))?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(wiring.spi_hz)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)
            .map_err(|error| format!("{}: {error}", wiring.spi_device.display()))?;
        let bus: Bus = ChunkedBus::new(spi, chunked_bus::SPIDEV_CHUNK);
        let mut chip: Chip = Chip::new(&wiring.gpio_chip)
            .map_err(|error| format!("{}: {error}", wiring.gpio_chip.display()))?;
        let power: Option<CdevPin> = match wiring.power {
            Some(line) => Some(output(&mut chip, line, PinState::Low)?),
            None => None,
        };
        let cs0: CdevPin = output(&mut chip, wiring.cs0, PinState::High)?;
        let dc: CdevPin = output(&mut chip, wiring.dc, PinState::Low)?;
        let rst: CdevPin = output(&mut chip, wiring.rst, PinState::High)?;
        let glass: Glassware = match panel {
            PanelSize::Inches13Point3 => {
                let cs1: CdevPin = output(&mut chip, wiring.cs1, PinState::High)?;
                let busy: CdevPin = input(&mut chip, wiring.busy)?;
                let pins = panel_t133a01::Pins {
                    cs0,
                    cs1,
                    dc,
                    rst,
                    busy,
                };
                Glassware::Large(T133a01::new(bus, pins, Delay))
            }
            PanelSize::Inches7Point3 => {
                let busy: CdevPin = input(&mut chip, wiring.busy)?;
                let pins = panel_ed2208::Pins {
                    cs: cs0,
                    dc,
                    rst,
                    busy,
                };
                Glassware::Small(Ed2208::new(bus, pins, Delay))
            }
        };
        Ok((glass, power))
    }

    fn output(chip: &mut Chip, line: u32, state: PinState) -> Result<CdevPin, String> {
        let level: u8 = u8::from(state == PinState::High);
        chip.get_line(line)
            .and_then(|line| line.request(LineRequestFlags::OUTPUT, level, "birdnet-go-frame"))
            .and_then(CdevPin::new)
            .map_err(|error| format!("GPIO line {line}: {error}"))
    }

    fn input(chip: &mut Chip, line: u32) -> Result<CdevPin, String> {
        chip.get_line(line)
            .and_then(|line| line.request(LineRequestFlags::INPUT, 0, "birdnet-go-frame"))
            .and_then(CdevPin::new)
            .map_err(|error| format!("GPIO line {line}: {error}"))
    }
}
