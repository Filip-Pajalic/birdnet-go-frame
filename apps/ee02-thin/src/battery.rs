//! The battery's voltage, through the EE02's divider on GPIO1.
//!
//! The pins are those of Seeed's EE0x boards (the divider on A0, GPIO1, switched on by GPIO6),
//! from Seeed's PlatformIO cookbook for the family.

use esp_hal::analog::adc::{Adc, AdcCalCurve, AdcConfig, AdcPin, Attenuation};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::{ADC1, GPIO1, GPIO6};

/// Readings taken; the median is kept.
const SAMPLES: usize = 9;
/// How long the divider settles after it is switched on.
const SETTLE_MS: u32 = 5;

/// Returns the battery's voltage in millivolts, or `None` when there is no battery.
pub fn read(adc: ADC1<'_>, pin: GPIO1<'_>, enable: GPIO6<'_>) -> Option<u16> {
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
