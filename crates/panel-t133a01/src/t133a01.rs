//! The driver.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{self, InputPin, OutputPin};
use embedded_hal::spi::{self, SpiBus};

use crate::commands::{self, Chips};
use crate::panel_error::PanelError;

/// Bytes in one page: 1200x1600 pixels, two a byte.
const PAGE_LEN: usize = 1200 * 1600 / 2;
/// Bytes sent between short pauses while a controller takes its half: 64 rows of 300 bytes.
const CHUNK: usize = 64 * 300;
/// How long BUSY may stay high after the refresh command before the refresh counts as not
/// started, in milliseconds.
const REFRESH_START_MS: u32 = 2_000;
/// The shortest a real refresh keeps BUSY low; a healthy one takes about 28 s, 47 s when cold.
const REFRESH_MIN_BUSY_MS: u32 = 10_000;
/// The longest a refresh may keep BUSY low.
const REFRESH_TIMEOUT_MS: u32 = 60_000;
/// How long the controllers may stay busy after an ordinary command.
const COMMAND_TIMEOUT_MS: u32 = 5_000;

/// The pins the panel needs besides the SPI bus's clock and data.
///
/// The panel's power switch and holding pins through deep sleep are the board's business, not
/// the driver's: power the panel before [`T133a01::wake`] and cut it after [`T133a01::sleep`].
#[derive(Debug)]
pub struct Pins<O, I> {
    /// Chip select of the first controller, active low.
    pub cs0: O,
    /// Chip select of the second controller, active low.
    pub cs1: O,
    /// Data/command: low for a command byte, high for its data.
    pub dc: O,
    /// Reset, active low.
    pub rst: O,
    /// BUSY from the controllers: low while working, high when ready.
    pub busy: I,
}

/// The 13.3" Spectra 6 panel on an SPI bus.
///
/// Times are counted in the delays the driver asks for, so they run a little short of the
/// wall clock; the limits leave room for that.
#[derive(Debug)]
pub struct T133a01<S, O, I, D> {
    spi: S,
    pins: Pins<O, I>,
    delay: D,
}

impl<S, O, I, D> T133a01<S, O, I, D>
where
    S: SpiBus,
    O: OutputPin,
    I: InputPin,
    D: DelayNs,
{
    /// The length [`draw`](Self::draw) takes: 960,000 bytes.
    pub const PAGE_LEN: usize = PAGE_LEN;

    /// Takes the bus (mode 0, up to 10 MHz), the pins and a delay. Nothing is sent yet.
    #[must_use]
    pub fn new(spi: S, pins: Pins<O, I>, delay: D) -> Self {
        Self { spi, pins, delay }
    }

    /// Gives the bus, pins and delay back, e.g. to hold the pins low through deep sleep.
    pub fn release(self) -> (S, Pins<O, I>, D) {
        (self.spi, self.pins, self.delay)
    }

    /// Resets the controllers, sends the start-up sequence and powers the panel on.
    ///
    /// # Errors
    ///
    /// Returns [`PanelError`] when the bus or a pin fails or the controllers stay busy.
    pub fn wake(&mut self) -> Result<(), PanelError> {
        set(&mut self.pins.cs0, true)?;
        set(&mut self.pins.cs1, true)?;
        set(&mut self.pins.dc, false)?;
        // Let a freshly powered panel settle, then a long reset: 100 ms each way.
        self.delay.delay_ms(100);
        set(&mut self.pins.rst, false)?;
        self.delay.delay_ms(100);
        set(&mut self.pins.rst, true)?;
        self.delay.delay_ms(100);
        self.wait()?;
        for (chips, command, data) in commands::INIT {
            self.command(chips, command, data)?;
            self.delay.delay_ms(10);
        }
        self.command(Chips::Both, commands::PLL, &[0x08])?;
        self.delay.delay_ms(10);
        // Power on before any pixel data.
        self.command(Chips::Both, commands::POWER_ON, &[])?;
        self.wait()?;
        self.delay.delay_ms(30);
        Ok(())
    }

    /// Sends a page: the first half of `page` to the first controller, the second half to the
    /// second. The glass does not change until [`refresh`](Self::refresh).
    ///
    /// # Errors
    ///
    /// Returns [`PanelError::WrongLength`] unless `page` is [`Self::PAGE_LEN`] bytes, or another
    /// [`PanelError`] when the bus or a pin fails or the controllers stay busy.
    pub fn draw(&mut self, page: &[u8]) -> Result<(), PanelError> {
        if page.len() != PAGE_LEN {
            return Err(PanelError::WrongLength(page.len()));
        }
        let (left, right) = page.split_at(PAGE_LEN / 2);
        for (chips, half) in [(Chips::Primary, left), (Chips::Secondary, right)] {
            self.command(Chips::Both, commands::CASCADE, &[0x01])?;
            self.wait()?;
            self.delay.delay_ms(10);
            self.select(chips, false)?;
            self.send(&[commands::DATA], false)?;
            for chunk in half.chunks(CHUNK) {
                self.send(chunk, true)?;
                self.delay.delay_ms(1);
            }
            self.select(chips, true)?;
            set(&mut self.pins.dc, false)?;
        }
        self.delay.delay_ms(30);
        Ok(())
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
        self.command(Chips::Both, commands::REFRESH, &[0x01])?;
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

    /// Powers the controllers off. The glass keeps its picture with no power at all.
    ///
    /// # Errors
    ///
    /// Returns [`PanelError`] when the bus or a pin fails or the controllers stay busy; cut the
    /// panel's power anyway.
    pub fn sleep(&mut self) -> Result<(), PanelError> {
        self.command(Chips::Both, commands::POWER_OFF, &[0x00])?;
        self.wait()?;
        self.delay.delay_ms(100);
        Ok(())
    }

    /// Sends `command` and its `data` to `chips`.
    fn command(&mut self, chips: Chips, command: u8, data: &[u8]) -> Result<(), PanelError> {
        self.select(chips, false)?;
        self.send(&[command], false)?;
        if !data.is_empty() {
            self.send(data, true)?;
        }
        self.select(chips, true)?;
        set(&mut self.pins.dc, false)
    }

    /// Writes `bytes` with DC at `is_data`, waiting until they are all out on the wire.
    fn send(&mut self, bytes: &[u8], is_data: bool) -> Result<(), PanelError> {
        set(&mut self.pins.dc, is_data)?;
        self.spi.write(bytes).map_err(bus)?;
        self.spi.flush().map_err(bus)
    }

    /// Sets the chip selects of `chips` to `high`.
    fn select(&mut self, chips: Chips, high: bool) -> Result<(), PanelError> {
        if chips.primary() {
            set(&mut self.pins.cs0, high)?;
        }
        if chips.secondary() {
            set(&mut self.pins.cs1, high)?;
        }
        Ok(())
    }

    /// Whether the controllers are working (BUSY low).
    fn busy(&mut self) -> Result<bool, PanelError> {
        self.pins.busy.is_low().map_err(pin)
    }

    /// Waits for the controllers to finish a command.
    fn wait(&mut self) -> Result<(), PanelError> {
        // The controllers take a moment to pull BUSY low after a command.
        self.delay.delay_ms(10);
        let mut waited: u32 = 0;
        while self.busy()? {
            if waited >= COMMAND_TIMEOUT_MS {
                return Err(PanelError::StayedBusy);
            }
            self.delay.delay_ms(50);
            waited += 50;
        }
        Ok(())
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
