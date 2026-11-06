//! What a panel remembers through deep sleep.

use frame_core::{Crc32, inflate_zlib};

use crate::response_head::ResponseHead;
use crate::wake::{SleepLimits, Wake};
use crate::wake_error::WakeError;

/// The longest ETag kept; the server's are 13 bytes. A longer one is not kept, which only costs
/// a full download next time.
const ETAG_MAX: usize = 32;

/// What the panel keeps between wakes: the ETag of the page on the glass and how many wakes in
/// a row have failed. Fixed-size and [`Copy`], to live in the ESP32's RTC memory.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WakeState {
    etag: [u8; ETAG_MAX],
    etag_len: u8,
    failures: u8,
}

impl WakeState {
    /// Bytes [`to_bytes`](Self::to_bytes) gives.
    pub const BYTES: usize = ETAG_MAX + 2;

    /// The ETag of the page on the glass, to send as `If-None-Match`.
    #[must_use]
    pub fn etag(&self) -> Option<&str> {
        let tag: &[u8] = self.etag.get(..usize::from(self.etag_len))?;
        core::str::from_utf8(tag).ok().filter(|tag| !tag.is_empty())
    }

    /// How many wakes in a row have ended without reaching a decision about the page.
    #[must_use]
    pub fn failures(&self) -> u8 {
        self.failures
    }

    /// Returns this state with `etag` as the page on the glass and no failures.
    #[must_use]
    pub fn with_etag(self, etag: Option<&str>) -> Self {
        let mut state = Self::default();
        if let Some(tag) = etag.filter(|tag| tag.len() <= ETAG_MAX) {
            state.etag[..tag.len()].copy_from_slice(tag.as_bytes());
            state.etag_len = tag.len() as u8;
        }
        state
    }

    /// The state as bytes, e.g. for RTC memory.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut bytes: [u8; Self::BYTES] = [0; Self::BYTES];
        bytes[..ETAG_MAX].copy_from_slice(&self.etag);
        bytes[ETAG_MAX] = self.etag_len;
        bytes[ETAG_MAX + 1] = self.failures;
        bytes
    }

    /// Reads [`to_bytes`](Self::to_bytes)' bytes. Memory that holds nothing valid, as after a
    /// power cut, gives the default state: no page known, so the next answer is drawn.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; Self::BYTES]) -> Self {
        let mut etag: [u8; ETAG_MAX] = [0; ETAG_MAX];
        etag.copy_from_slice(&bytes[..ETAG_MAX]);
        let state = Self {
            etag,
            etag_len: bytes[ETAG_MAX],
            failures: bytes[ETAG_MAX + 1],
        };
        let valid: bool = usize::from(state.etag_len) <= ETAG_MAX
            && (state.etag_len == 0 || state.etag().is_some());
        if valid { state } else { Self::default() }
    }

    /// Decides what to do with the server's answer. A new page is unpacked into `page`, which
    /// must be the panel's size, and checked; [`Wake::show`] says whether to draw it.
    pub fn after_response(
        self,
        head: &ResponseHead,
        body: &[u8],
        page: &mut [u8],
        limits: SleepLimits,
    ) -> Wake {
        let told: Option<u32> = head.max_age;
        match head.status {
            304 => Wake {
                show: false,
                sleep_seconds: limits.clamp(told),
                state: Self {
                    failures: 0,
                    ..self
                },
                error: None,
            },
            200 => match unpack(head, body, page) {
                Ok(()) => Wake {
                    show: true,
                    sleep_seconds: limits.clamp(told),
                    state: self.with_etag(head.etag.as_deref()),
                    error: None,
                },
                Err(error) => self.failed(error, limits),
            },
            status => self.failed(WakeError::Status(status), limits),
        }
    }

    /// The wake ended before any answer: no Wi-Fi, no server, a broken connection.
    pub fn after_network_failure(self, limits: SleepLimits) -> Wake {
        self.failed(WakeError::Network, limits)
    }

    /// Keeps the page on the glass and backs off: each failure in a row doubles the sleep.
    fn failed(self, error: WakeError, limits: SleepLimits) -> Wake {
        let failures: u8 = self.failures.saturating_add(1);
        Wake {
            show: false,
            sleep_seconds: limits.backoff(failures),
            state: Self { failures, ..self },
            error: Some(error),
        }
    }
}

/// Unpacks the body into `page` and checks its length and CRC-32.
fn unpack(head: &ResponseHead, body: &[u8], page: &mut [u8]) -> Result<(), WakeError> {
    let length: usize = if head.deflate {
        match inflate_zlib(body, page) {
            Ok(length) => length,
            Err(frame_core::InflateError::TooLong) => {
                return Err(WakeError::WrongLength {
                    expected: page.len(),
                    got: page.len() + 1,
                });
            }
            Err(error) => return Err(WakeError::Inflate(error)),
        }
    } else {
        if let Some(target) = page.get_mut(..body.len()) {
            target.copy_from_slice(body);
        }
        body.len()
    };
    if length != page.len() {
        return Err(WakeError::WrongLength {
            expected: page.len(),
            got: length,
        });
    }
    match head.page_crc() {
        Some(crc) if crc != Crc32::checksum(page) => Err(WakeError::Damaged),
        _ => Ok(()),
    }
}
