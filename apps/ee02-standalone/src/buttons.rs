//! The board's three buttons: which one woke the frame, and waking on them again.
//!
//! The pins are in `board.rs`, the same as `ee02-thin`'s; on both boards the buttons are pulled
//! up and low when pressed. What a press does is `frame-standalone`'s
//! (`StandaloneMemory::after_press`): previous and next step through every page, refresh draws
//! the page again.

use esp_hal::gpio::{Event, Input, InputConfig, InputPin, Pull, WakeupConfig};
use esp_println::println;
use panel_wake::Button;

/// The three buttons, as inputs with pull-ups.
pub struct Buttons<'d> {
    refresh: Input<'d>,
    previous: Input<'d>,
    next: Input<'d>,
}

impl<'d> Buttons<'d> {
    /// Takes the buttons' pins.
    pub fn new(
        refresh: impl InputPin + 'd,
        previous: impl InputPin + 'd,
        next: impl InputPin + 'd,
    ) -> Self {
        let config: InputConfig = InputConfig::default().with_pull(Pull::Up);
        Self {
            refresh: Input::new(refresh, config),
            previous: Input::new(previous, config),
            next: Input::new(next, config),
        }
    }

    /// Returns the button that ended the deep sleep, if one did.
    pub fn pressed(&self) -> Option<Button> {
        if self.previous.caused_wakeup() {
            Some(Button::Previous)
        } else if self.next.caused_wakeup() {
            Some(Button::Next)
        } else if self.refresh.caused_wakeup() {
            Some(Button::Refresh)
        } else {
            None
        }
    }

    /// Lets a press of any button end the coming deep sleep. Keep the buttons until the sleep:
    /// dropping them stops them listening.
    pub fn arm(&mut self) {
        let config: WakeupConfig = WakeupConfig::default().with_low_power_path(true);
        for button in [&mut self.refresh, &mut self.previous, &mut self.next] {
            button.listen(Event::LowLevel);
            if let Err(error) = button.apply_wakeup_config(&config) {
                println!("standalone: a button cannot wake the frame: {error:?}");
            }
        }
    }
}
