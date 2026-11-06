//! The outcome of a wake.

use crate::wake_error::WakeError;
use crate::wake_state::WakeState;

/// What to do at the end of a wake.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub struct Wake {
    /// Whether to draw the page just unpacked and refresh the glass.
    pub show: bool,
    /// How long to deep-sleep.
    pub sleep_seconds: u32,
    /// What to keep in RTC memory for the next wake.
    pub state: WakeState,
    /// Why there is no page, when the wake failed; the glass keeps its picture.
    pub error: Option<WakeError>,
}

/// Bounds on the panel's sleep, whatever the server says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SleepLimits {
    /// The shortest sleep: a server asking for less (or `no-cache`) gets this.
    pub min_seconds: u32,
    /// The longest sleep, also the cap on backing off after failures.
    pub max_seconds: u32,
    /// The sleep when the server gives no `max-age`.
    pub default_seconds: u32,
}

impl Default for SleepLimits {
    /// Five minutes at least, twelve hours at most, fifteen minutes when not told.
    fn default() -> Self {
        Self {
            min_seconds: 5 * 60,
            max_seconds: 12 * 60 * 60,
            default_seconds: 15 * 60,
        }
    }
}

impl SleepLimits {
    /// The sleep for a server's `max-age`, within the limits.
    #[must_use]
    pub fn clamp(self, max_age: Option<u32>) -> u32 {
        max_age
            .unwrap_or(self.default_seconds)
            .clamp(self.min_seconds, self.max_seconds.max(self.min_seconds))
    }

    /// The sleep after `failures` failed wakes in a row: the shortest, doubled for each failure
    /// after the first, up to the longest.
    #[must_use]
    pub fn backoff(self, failures: u8) -> u32 {
        let doublings: u32 = u32::from(failures.saturating_sub(1)).min(16);
        self.min_seconds
            .saturating_mul(1 << doublings)
            .min(self.max_seconds.max(self.min_seconds))
    }
}
