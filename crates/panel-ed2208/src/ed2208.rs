//! The driver.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{self, InputPin, OutputPin};
use embedded_hal::spi::{self, SpiBus};
use frame_core::{Glass, PanelSize};

use crate::commands;
use crate::panel_error::PanelError;

/// Bytes in one page: 800x480 pixels, two a byte. [`Ed2208::draw`] takes exactly this many.
pub const PAGE_LEN: usize = 800 * 480 / 2;
/// Bytes sent between short pauses while the controller takes the page: 16 rows of 400 bytes.
const CHUNK: usize = 16 * 400;
/// How long BUSY may stay high after the refresh command before the refresh counts as not
/// started, in milliseconds.
const REFRESH_START_MS: u32 = 2_000;
/// The shortest a real refresh keeps BUSY low; a healthy one takes about 20 s.
const REFRESH_MIN_BUSY_MS: u32 = 5_000;
/// The longest a refresh may keep BUSY low.
const REFRESH_TIMEOUT_MS: u32 = 60_000;
/// How long the controller may stay busy after an ordinary command.
const COMMAND_TIMEOUT_MS: u32 = 5_000;

/// The pins the panel needs besides the SPI bus's clock and data.
///
/// Any power switch for the panel, and holding pins through deep sleep, are the board's
/// business: power the panel before [`Ed2208::wake`] and cut it after [`Ed2208::sleep`].
#[derive(Debug)]
pub struct Pins<O, I> {
    /// Chip select, active low.
    pub cs: O,
    /// Data/command: low for a command byte, high for its data.
    pub dc: O,
    /// Reset, active low.
    pub rst: O,
    /// BUSY from the controller: low while working, high when ready.
    pub busy: I,
}

/// The 7.3" Spectra 6 panel on an SPI bus.
///
/// Times are counted in the delays the driver asks for, so they run a little short of the
/// wall clock; the limits leave room for that.
#[derive(Debug)]
pub struct Ed2208<S, O, I, D> {
    spi: S,
    pins: Pins<O, I>,
    delay: D,
}

impl<S, O, I, D> Ed2208<S, O, I, D>
where
    S: SpiBus,
    O: OutputPin,
    I: InputPin,
    D: DelayNs,
{
    /// The length [`draw`](Self::draw) takes: 192,000 bytes.
    pub const PAGE_LEN: usize = PAGE_LEN;

    /// Takes the bus (mode 0, up to 4 MHz as Seeed's reTerminal E1002 runs it), the pins and a
    /// delay. Nothing is sent yet.
    #[must_use]
    pub fn new(spi: S, pins: Pins<O, I>, delay: D) -> Self {
        Self { spi, pins, delay }
    }

    /// Gives the bus, pins and delay back, e.g. to hold the pins low through deep sleep.
    pub fn release(self) -> (S, Pins<O, I>, D) {
        (self.spi, self.pins, self.delay)
    }

    /// Resets the controller, sends the start-up sequence and powers the panel on. Also wakes
    /// the controller from [`sleep`](Self::sleep), which only a reset can.
    ///
    /// # Errors
    ///
    /// Returns [`PanelError`] when the bus or a pin fails or the controller stays busy.
    pub fn wake(&mut self) -> Result<(), PanelError> {
        set(&mut self.pins.cs, true)?;
        set(&mut self.pins.dc, false)?;
        set(&mut self.pins.rst, false)?;
        self.delay.delay_ms(20);
        set(&mut self.pins.rst, true)?;
        self.delay.delay_ms(20);
        self.wait()?;
        for (command, data) in commands::INIT {
            self.command(command, data)?;
        }
        self.command(commands::POWER_ON, &[])?;
        self.wait()
    }

    /// Sends a page. The glass does not change until [`refresh`](Self::refresh).
    ///
    /// # Errors
    ///
    /// Returns [`PanelError::WrongLength`] unless `page` is [`Self::PAGE_LEN`] bytes, or another
    /// [`PanelError`] when the bus or a pin fails.
    pub fn draw(&mut self, page: &[u8]) -> Result<(), PanelError> {
        if page.len() != PAGE_LEN {
            return Err(PanelError::WrongLength(page.len()));
        }
        set(&mut self.pins.cs, false)?;
        self.send(&[commands::DATA], false)?;
        for chunk in page.chunks(CHUNK) {
            self.send(chunk, true)?;
        }
        set(&mut self.pins.cs, true)?;
        set(&mut self.pins.dc, false)
    }

    /// Refreshes the glass with what was drawn and follows BUSY through it. Returns how many
    /// milliseconds the refresh took.
    ///
    /// # Errors
    ///
    /// Returns [`PanelError::RefreshNotStarted`], [`PanelError::RefreshTimedOut`] or
    /// [`PanelError::RefreshTooShort`] when BUSY does not behave like a real refresh, or another
    /// [`PanelError`] when the bus or a pin fails.
    pub fn refresh(&mut self) -> Result<u32, PanelError> {
        self.command(commands::REFRESH, &[0x00])?;
        let mut waited: u32 = 0;
        while !self.busy()? {
            if waited >= REFRESH_START_MS {
                return Err(PanelError::RefreshNotStarted);
            }
            self.delay.delay_ms(2);
            waited += 2;
        }
        let mut busy: u32 = 0;
        while self.busy()? {
            if busy >= REFRESH_TIMEOUT_MS {
                return Err(PanelError::RefreshTimedOut);
            }
            self.delay.delay_ms(5);
            busy += 5;
        }
        if busy < REFRESH_MIN_BUSY_MS {
            return Err(PanelError::RefreshTooShort(busy));
        }
        Ok(busy)
    }

    /// Draws `page` and refreshes the glass. Returns how many milliseconds the refresh took.
    ///
    /// # Errors
    ///
    /// As [`draw`](Self::draw) and [`refresh`](Self::refresh).
    pub fn show(&mut self, page: &[u8]) -> Result<u32, PanelError> {
        self.draw(page)?;
        self.refresh()
    }

    /// Powers the panel off and puts the controller into deep sleep. The glass keeps its
    /// picture with no power at all; [`wake`](Self::wake) resets the controller out of it.
    ///
    /// # Errors
    ///
    /// Returns [`PanelError`] when the bus or a pin fails or the controller stays busy; cut the
    /// panel's power anyway, if the board can.
    pub fn sleep(&mut self) -> Result<(), PanelError> {
        self.command(commands::POWER_OFF, &[0x00])?;
        self.wait()?;
        self.command(commands::DEEP_SLEEP, &[commands::DEEP_SLEEP_CHECK])
    }

    /// Sends `command` and its `data`.
    fn command(&mut self, command: u8, data: &[u8]) -> Result<(), PanelError> {
        set(&mut self.pins.cs, false)?;
        self.send(&[command], false)?;
        if !data.is_empty() {
            self.send(data, true)?;
        }
        set(&mut self.pins.cs, true)?;
        set(&mut self.pins.dc, false)
    }

    /// Writes `bytes` with DC at `is_data`, waiting until they are all out on the wire.
    fn send(&mut self, bytes: &[u8], is_data: bool) -> Result<(), PanelError> {
        set(&mut self.pins.dc, is_data)?;
        self.spi.write(bytes).map_err(bus)?;
        self.spi.flush().map_err(bus)
    }

    /// Whether the controller is working (BUSY low).
    fn busy(&mut self) -> Result<bool, PanelError> {
        self.pins.busy.is_low().map_err(pin)
    }

    /// Waits for the controller to finish a command.
    fn wait(&mut self) -> Result<(), PanelError> {
        self.delay.delay_ms(1);
        let mut waited: u32 = 0;
        while self.busy()? {
            if waited >= COMMAND_TIMEOUT_MS {
                return Err(PanelError::StayedBusy);
            }
            self.delay.delay_ms(10);
            waited += 10;
        }
        Ok(())
    }
}

impl<S, O, I, D> Glass for Ed2208<S, O, I, D>
where
    S: SpiBus,
    O: OutputPin,
    I: InputPin,
    D: DelayNs,
{
    type Error = PanelError;

    const PANEL: PanelSize = PanelSize::Inches7Point3;

    fn wake(&mut self) -> Result<(), PanelError> {
        Ed2208::wake(self)
    }

    fn show(&mut self, page: &[u8]) -> Result<u32, PanelError> {
        Ed2208::show(self, page)
    }

    fn sleep(&mut self) -> Result<(), PanelError> {
        Ed2208::sleep(self)
    }
}

fn set(pin: &mut impl OutputPin, high: bool) -> Result<(), PanelError> {
    if high { pin.set_high() } else { pin.set_low() }.map_err(self::pin)
}

fn bus(error: impl spi::Error) -> PanelError {
    PanelError::Bus(error.kind())
}

fn pin(error: impl digital::Error) -> PanelError {
    PanelError::Pin(error.kind())
}
