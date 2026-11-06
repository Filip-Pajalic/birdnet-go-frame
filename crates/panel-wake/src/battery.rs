//! The battery's voltage, from the ADC's readings.

/// A battery reading below this is no battery: the board runs from USB, or the switch is off.
const NO_BATTERY_MILLIVOLTS: u16 = 2_500;

/// Below this, the panel skips firmware updates: a brown-out while writing flash costs one
/// boot of the old firmware, but there is no reason to risk it.
pub const UPDATE_MIN_MILLIVOLTS: u16 = 3_600;

/// Returns the battery's voltage from millivolt readings at the ADC pin, behind a divider that
/// halves it (the EE02's two equal resistors).
///
/// Takes the median, so one noisy reading does not move the result, and returns `None` when
/// there are no readings or they show no battery.
///
/// ```
/// let mut samples: [u16; 5] = [1_950, 1_960, 4_000, 1_955, 1_940];
/// assert_eq!(panel_wake::battery_millivolts(&mut samples), Some(3_910));
/// assert_eq!(panel_wake::battery_millivolts(&mut [0, 3, 1]), None);
/// ```
#[must_use]
pub fn battery_millivolts(samples: &mut [u16]) -> Option<u16> {
    if samples.is_empty() {
        return None;
    }
    samples.sort_unstable();
    let median: u16 = samples[samples.len() / 2];
    let battery: u16 = median.saturating_mul(2);
    (battery >= NO_BATTERY_MILLIVOLTS).then_some(battery)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ignores_noise_and_an_absent_battery() {
        assert_eq!(battery_millivolts(&mut []), None);
        assert_eq!(battery_millivolts(&mut [2_100]), Some(4_200));
        assert_eq!(
            battery_millivolts(&mut [1_800, 0, 1_801, 1_799]),
            Some(3_600)
        );
        assert_eq!(battery_millivolts(&mut [1_249, 1_249, 1_249]), None);
        assert_eq!(battery_millivolts(&mut [1_250, 1_250, 1_250]), Some(2_500));
    }
}
