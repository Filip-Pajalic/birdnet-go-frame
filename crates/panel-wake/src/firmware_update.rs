//! When the panel installs the firmware the server offers.

use frame_core::Crc32;

use crate::battery::UPDATE_MIN_MILLIVOLTS;

/// Failed tries at one offered version before the panel stops trying it. A newer offer starts
/// over.
const MAX_TRIES: u8 = 3;

/// What the panel remembers about updating through deep sleep: which offered version failed to
/// install and how many times.
///
/// ```
/// use panel_wake::FirmwareUpdate;
///
/// let update = FirmwareUpdate::default();
/// assert!(update.wanted("0.1.0", Some("0.2.0"), Some(3_900)));
/// assert!(!update.wanted("0.2.0", Some("0.2.0"), Some(3_900)));
/// assert!(!update.wanted("0.1.0", Some("0.2.0"), Some(3_400)));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FirmwareUpdate {
    /// CRC-32 of the version that failed.
    version: u32,
    /// Failed tries at it.
    failures: u8,
}

impl FirmwareUpdate {
    /// Bytes [`to_bytes`](Self::to_bytes) gives.
    pub const BYTES: usize = 5;

    /// Whether to download `offered` while `running` runs, on a battery at `battery_millivolts`
    /// (`None` on USB power).
    ///
    /// Any version other than the running one is installed, so the server can roll a frame back
    /// as well as forward. A low battery waits, and a version that failed to install three times
    /// is left alone until the server offers another.
    #[must_use]
    pub fn wanted(
        &self,
        running: &str,
        offered: Option<&str>,
        battery_millivolts: Option<u16>,
    ) -> bool {
        let Some(offered) = offered.map(str::trim).filter(|offered| !offered.is_empty()) else {
            return false;
        };
        let charged: bool =
            battery_millivolts.is_none_or(|millivolts| millivolts >= UPDATE_MIN_MILLIVOLTS);
        let given_up: bool =
            self.version == Crc32::checksum(offered.as_bytes()) && self.failures >= MAX_TRIES;
        offered != running && charged && !given_up
    }

    /// Returns this record after a failed try at `offered`.
    #[must_use]
    pub fn failed(self, offered: &str) -> Self {
        let version: u32 = Crc32::checksum(offered.trim().as_bytes());
        let failures: u8 = if version == self.version {
            self.failures
        } else {
            0
        };
        Self {
            version,
            failures: failures.saturating_add(1),
        }
    }

    /// The record as bytes, e.g. for RTC memory.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut bytes: [u8; Self::BYTES] = [0; Self::BYTES];
        bytes[..4].copy_from_slice(&self.version.to_le_bytes());
        bytes[4] = self.failures;
        bytes
    }

    /// Reads [`to_bytes`](Self::to_bytes)' bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; Self::BYTES]) -> Self {
        Self {
            version: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            failures: bytes[4],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gives_up_on_a_version_after_three_failures() {
        let mut update = FirmwareUpdate::default();
        for _ in 0..3 {
            assert!(update.wanted("0.1.0", Some("0.2.0"), None));
            update = update.failed("0.2.0");
        }
        assert!(!update.wanted("0.1.0", Some("0.2.0"), None));
        assert!(update.wanted("0.1.0", Some(" 0.2.1 "), None));
        assert_eq!(update.failed("0.2.1").failed("0.2.1").failures, 2);
        assert_eq!(FirmwareUpdate::from_bytes(&update.to_bytes()), update);
    }

    #[test]
    fn needs_an_offer_and_a_charge() {
        let update = FirmwareUpdate::default();
        assert!(!update.wanted("0.1.0", None, None));
        assert!(!update.wanted("0.1.0", Some("  "), None));
        assert!(update.wanted("0.2.0", Some("0.1.0"), Some(UPDATE_MIN_MILLIVOLTS)));
        assert!(!update.wanted("0.2.0", Some("0.1.0"), Some(UPDATE_MIN_MILLIVOLTS - 1)));
    }
}
