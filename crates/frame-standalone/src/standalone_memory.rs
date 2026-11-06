//! What a standalone frame keeps through deep sleep, and how long it sleeps.

use birdnet_model::SpeciesSummary;
use frame_artist::Layout;
use frame_core::Crc32;
use panel_wake::Rejoin;

use crate::fingerprint::Fingerprint;

/// Bytes the memory takes before its checksum: flags, failures, two spare, the fingerprint,
/// then the rejoin record.
const BYTES: usize = 8 + Rejoin::BYTES;
/// Those bytes in whole words.
const STATE_WORDS: usize = BYTES.div_ceil(4);

/// What a standalone frame remembers between wakes: the page on the glass, the failures in a
/// row, and how to join the Wi-Fi quickly. It fits in a few words of RTC memory.
///
/// ```
/// use frame_standalone::{Fingerprint, StandaloneMemory};
///
/// let memory = StandaloneMemory::default().after_shown(Fingerprint(7));
/// let words: [u32; StandaloneMemory::WORDS] = memory.to_words();
/// assert_eq!(StandaloneMemory::from_words(&words), memory);
/// assert_eq!(StandaloneMemory::from_words(&[0; StandaloneMemory::WORDS]), StandaloneMemory::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StandaloneMemory {
    /// The fingerprint of the page on the glass, if one was drawn.
    pub shown: Option<Fingerprint>,
    /// Wakes in a row that failed to reach BirdNET-Go or to draw.
    pub failures: u8,
    /// How to join the Wi-Fi quickly, if the last wake reached the network.
    pub rejoin: Option<Rejoin>,
}

/// What a wake decided after asking BirdNET-Go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StandaloneWake {
    /// Whether the page changed and must be drawn.
    pub show: bool,
    /// The new page's fingerprint, for [`StandaloneMemory::after_shown`].
    pub fingerprint: Fingerprint,
    /// How long to sleep afterwards.
    pub sleep_seconds: u32,
}

impl StandaloneMemory {
    /// Words [`to_words`](Self::to_words) gives: the state, then its CRC-32.
    pub const WORDS: usize = STATE_WORDS + 1;

    /// Decides what to do with the birds BirdNET-Go sent: draw them if the page they make on
    /// `day` (days since 1970) in `layout` is not the one on the glass, and sleep `poll_minutes`
    /// (at least one).
    #[must_use]
    pub fn after_species(
        &self,
        day: i64,
        species: &[SpeciesSummary],
        layout: Layout,
        poll_minutes: u16,
    ) -> StandaloneWake {
        let fingerprint: Fingerprint = Fingerprint::of(day, species, layout);
        StandaloneWake {
            show: self.shown != Some(fingerprint),
            fingerprint,
            sleep_seconds: u32::from(poll_minutes.max(1)) * 60,
        }
    }

    /// Returns the memory after the page with `fingerprint` was drawn, or was already there.
    #[must_use]
    pub fn after_shown(self, fingerprint: Fingerprint) -> Self {
        Self {
            shown: Some(fingerprint),
            failures: 0,
            ..self
        }
    }

    /// Returns the memory after a wake that could not reach BirdNET-Go or draw, and how long to
    /// sleep: a minute after the first failure, doubling after each one in a row, but never
    /// longer than `poll_minutes`. A failed drawing forgets the page, so the next wake draws it
    /// again.
    #[must_use]
    pub fn after_failure(self, poll_minutes: u16, drawing: bool) -> (Self, u32) {
        let failures: u8 = self.failures.saturating_add(1);
        let backoff: u32 = 60u32 << u32::from(failures.min(12) - 1);
        let sleep: u32 = backoff.min(u32::from(poll_minutes.max(1)) * 60);
        let memory = Self {
            shown: if drawing { None } else { self.shown },
            failures,
            ..self
        };
        (memory, sleep)
    }

    /// The memory as little-endian words, the last one a CRC-32 of the rest.
    #[must_use]
    pub fn to_words(&self) -> [u32; Self::WORDS] {
        let mut bytes: [u8; STATE_WORDS * 4] = [0; STATE_WORDS * 4];
        bytes[0] = u8::from(self.shown.is_some()) | (u8::from(self.rejoin.is_some()) << 1);
        bytes[1] = self.failures;
        if let Some(Fingerprint(fingerprint)) = self.shown {
            bytes[4..8].copy_from_slice(&fingerprint.to_le_bytes());
        }
        if let Some(rejoin) = self.rejoin {
            bytes[8..BYTES].copy_from_slice(&rejoin.to_bytes());
        }
        let mut words: [u32; Self::WORDS] = [0; Self::WORDS];
        for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(4)) {
            *word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        words[STATE_WORDS] = Crc32::checksum(&bytes);
        words
    }

    /// Reads [`to_words`](Self::to_words)' words. Words whose checksum does not match, as after
    /// a first boot, give the default: nothing on the glass, so the next wake draws.
    #[must_use]
    pub fn from_words(words: &[u32; Self::WORDS]) -> Self {
        let mut bytes: [u8; STATE_WORDS * 4] = [0; STATE_WORDS * 4];
        for (chunk, word) in bytes.chunks_exact_mut(4).zip(words) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        if Crc32::checksum(&bytes) != words[STATE_WORDS] {
            return Self::default();
        }
        let mut rejoin: [u8; Rejoin::BYTES] = [0; Rejoin::BYTES];
        rejoin.copy_from_slice(&bytes[8..BYTES]);
        Self {
            shown: (bytes[0] & 1 == 1)
                .then(|| Fingerprint(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))),
            failures: bytes[1],
            rejoin: if bytes[0] & 2 == 2 {
                Rejoin::from_bytes(&rejoin)
            } else {
                None
            },
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use super::*;

    #[test]
    fn draws_only_a_changed_page() {
        let species: Vec<SpeciesSummary> =
            SpeciesSummary::parse_list(br#"[{"scientific_name":"Parus major","count":3}]"#)
                .expect("valid JSON");
        let first: StandaloneWake =
            StandaloneMemory::default().after_species(10, &species, Layout::Collage, 15);
        assert!(first.show);
        assert_eq!(first.sleep_seconds, 900);
        let memory: StandaloneMemory = StandaloneMemory::default().after_shown(first.fingerprint);
        assert!(!memory.after_species(10, &species, Layout::Collage, 15).show);
        assert!(memory.after_species(11, &species, Layout::Collage, 15).show);
    }

    #[test]
    fn backs_off_after_failures_up_to_the_poll() {
        let mut memory: StandaloneMemory = StandaloneMemory::default().after_shown(Fingerprint(1));
        let mut sleeps: Vec<u32> = Vec::new();
        for _ in 0..6 {
            let (next, sleep) = memory.after_failure(15, false);
            sleeps.push(sleep);
            memory = next;
        }
        assert_eq!(sleeps, [60, 120, 240, 480, 900, 900]);
        assert_eq!(memory.shown, Some(Fingerprint(1)));
        let (forgot, _) = memory.after_failure(15, true);
        assert_eq!(forgot.shown, None);
        assert_eq!(forgot.after_shown(Fingerprint(2)).failures, 0);
        let (many, sleep) = StandaloneMemory {
            failures: u8::MAX,
            ..StandaloneMemory::default()
        }
        .after_failure(600, false);
        assert_eq!((many.failures, sleep), (u8::MAX, 36_000));
    }
}
