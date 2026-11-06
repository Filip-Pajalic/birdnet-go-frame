//! The battery's voltage, through the board's divider on GPIO1.
//!
//! On the EE02 the divider is switched on by GPIO6 (Seeed's PlatformIO cookbook for the EE0x
//! family), on the E1002 by GPIO21 (see `board.rs`).

use esp_hal::analog::adc::{Adc, AdcCalCurve, AdcConfig, AdcPin, Attenuation};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};
use esp_hal::peripherals::{ADC1, GPIO1};

/// Readings taken; the median is kept.
const SAMPLES: usize = 9;
/// How long the divider settles after it is switched on.
const SETTLE_MS: u32 = 5;

/// Returns the battery's voltage in millivolts, or `None` when there is no battery.
pub fn read<'d>(adc: ADC1<'d>, pin: GPIO1<'d>, enable: impl OutputPin + 'd) -> Option<u16> {
    let mut switch: Output<'_> = Output::new(enable, Level::Low, OutputConfig::default());
    // The last sleep held the switch off; let go of it.
    switch.set_pad_hold(false);
    switch.set_high();
    Delay::new().delay_millis(SETTLE_MS);
    let mut config: AdcConfig<ADC1<'_>> = AdcConfig::new();
    let mut input: AdcPin<GPIO1<'_>, ADC1<'_>, AdcCalCurve<ADC1<'_>>> =
        config.enable_pin_with_cal(pin, Attenuation::_11dB);
    let mut adc = Adc::new(adc, config);
    let mut samples: [u16; SAMPLES] = [0; SAMPLES];
    for sample in &mut samples {
        *sample = adc.read_blocking(&mut input);
    }
    // Off through the sleep, so the divider does not drain the battery.
    switch.set_low();
    switch.set_pad_hold(true);
    panel_wake::battery_millivolts(&mut samples)
}
