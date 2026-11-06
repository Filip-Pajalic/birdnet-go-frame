//! The EE02's three buttons: which one woke the frame, and waking on them again.
//!
//! The pins are those of Seeed's EE0x boards (KEY1 GPIO2, KEY2 GPIO3, KEY3 GPIO5, all pulled
//! up and low when pressed), from Seeed's PlatformIO cookbook for the family.

use esp_hal::delay::Delay;
use esp_hal::gpio::{Event, Input, InputConfig, Pull, WakeupConfig};
use esp_hal::peripherals::{GPIO2, GPIO3, GPIO5};
use esp_println::println;
use panel_wake::Button;

/// How long the refresh button must be held at power-up to open the setup hotspot.
const HOLD_FOR_SETUP_MS: u32 = 3_000;
/// How often the held button is looked at.
const POLL_MS: u32 = 50;

/// The three buttons, as inputs with pull-ups.
pub struct Buttons<'d> {
    refresh: Input<'d>,
    previous: Input<'d>,
    next: Input<'d>,
}

/// What the buttons asked for at power-up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asked {
    /// Nothing: a timer wake or a plain power-up.
    Nothing,
    /// A short press of a button woke the frame.
    Pressed(Button),
    /// The refresh button was held: open the setup hotspot.
    Setup,
}

impl<'d> Buttons<'d> {
    /// Takes the buttons' pins.
    pub fn new(refresh: GPIO2<'d>, previous: GPIO3<'d>, next: GPIO5<'d>) -> Self {
        let config: InputConfig = InputConfig::default().with_pull(Pull::Up);
        Self {
            refresh: Input::new(refresh, config),
            previous: Input::new(previous, config),
            next: Input::new(next, config),
        }
    }

    /// Returns what the buttons ask for: the button that ended the deep sleep, or the refresh
    /// button held down at power-up or reset for three seconds.
    pub fn asked(&self) -> Asked {
        if self.previous.caused_wakeup() {
            return Asked::Pressed(Button::Previous);
        }
        if self.next.caused_wakeup() {
            return Asked::Pressed(Button::Next);
        }
        let woke: bool = self.refresh.caused_wakeup();
        if !woke && self.refresh.is_high() {
            return Asked::Nothing;
        }
        let delay = Delay::new();
        let mut held: u32 = 0;
        while self.refresh.is_low() && held < HOLD_FOR_SETUP_MS {
            delay.delay_millis(POLL_MS);
            held += POLL_MS;
        }
        if held >= HOLD_FOR_SETUP_MS {
            println!("ee02: refresh held, opening the setup hotspot");
            Asked::Setup
        } else if woke {
            Asked::Pressed(Button::Refresh)
        } else {
            Asked::Nothing
        }
    }

    /// Lets a press of any button end the coming deep sleep. Keep the buttons until the sleep:
    /// dropping them stops them listening.
    pub fn arm(&mut self) {
        let config: WakeupConfig = WakeupConfig::default().with_low_power_path(true);
        for button in [&mut self.refresh, &mut self.previous, &mut self.next] {
            button.listen(Event::LowLevel);
            if let Err(error) = button.apply_wakeup_config(&config) {
                println!("ee02: a button cannot wake the frame: {error:?}");
            }
        }
    }
}
