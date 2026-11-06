//! Everything the panel keeps through deep sleep, as words for RTC memory.

use frame_core::Crc32;

use crate::button::Button;
use crate::firmware_update::FirmwareUpdate;
use crate::page_view::PageView;
use crate::rejoin::Rejoin;
use crate::wake_state::WakeState;

/// Where each part sits in the bytes before the checksum.
const REJOIN_FLAG: usize = WakeState::BYTES;
const REJOIN_AT: usize = REJOIN_FLAG + 1;
const VIEW_AT: usize = REJOIN_AT + Rejoin::BYTES;
const UPDATE_AT: usize = VIEW_AT + 1;
/// Bytes of state before the checksum.
const STATE_BYTES: usize = UPDATE_AT + FirmwareUpdate::BYTES;
/// Words of state, rounded up.
const STATE_WORDS: usize = STATE_BYTES.div_ceil(4);

/// What the panel keeps between wakes: the [`WakeState`], after a good wake the [`Rejoin`]
/// record, the [`PageView`] the buttons chose, and the [`FirmwareUpdate`] record.
///
/// RTC memory can hold anything after a power cut or a reset in the middle of a write, so the
/// words carry a CRC-32 and memory that fails it reads as a first boot.
///
/// ```
/// use panel_wake::{SleepMemory, WakeState};
///
/// let memory = SleepMemory {
///     wake: WakeState::default().with_etag(Some("\"b35332bb-z\"")),
///     ..SleepMemory::default()
/// };
/// let words: [u32; SleepMemory::WORDS] = memory.to_words();
/// assert_eq!(SleepMemory::from_words(&words), memory);
/// assert_eq!(SleepMemory::from_words(&[0; SleepMemory::WORDS]), SleepMemory::default());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SleepMemory {
    /// The page on the glass and the failures in a row.
    pub wake: WakeState,
    /// How to join the Wi-Fi quickly, if the last wake reached the server.
    pub rejoin: Option<Rejoin>,
    /// The page to ask for.
    pub view: PageView,
    /// Failed tries at the firmware the server offers.
    pub update: FirmwareUpdate,
}

impl SleepMemory {
    /// Words [`to_words`](Self::to_words) gives: the state, then its CRC-32.
    pub const WORDS: usize = STATE_WORDS + 1;

    /// Returns the memory after `button` woke the panel: [`Button::Refresh`] forgets the page on
    /// the glass, so the next answer is drawn whether it changed or not; the other two step the
    /// view. Either way the failures in a row are forgotten, so the wake is not a long back-off.
    #[must_use]
    pub fn after_press(self, button: Button) -> Self {
        match button {
            Button::Refresh => Self {
                wake: WakeState::default(),
                ..self
            },
            Button::Previous => Self {
                wake: self.wake.with_etag(self.wake.etag()),
                view: self.view.previous(),
                ..self
            },
            Button::Next => Self {
                wake: self.wake.with_etag(self.wake.etag()),
                view: self.view.next(),
                ..self
            },
        }
    }

    /// The memory as little-endian words, the last one a CRC-32 of the rest.
    #[must_use]
    pub fn to_words(&self) -> [u32; Self::WORDS] {
        let mut bytes: [u8; STATE_WORDS * 4] = [0; STATE_WORDS * 4];
        bytes[..WakeState::BYTES].copy_from_slice(&self.wake.to_bytes());
        if let Some(rejoin) = self.rejoin {
            bytes[REJOIN_FLAG] = 1;
            bytes[REJOIN_AT..VIEW_AT].copy_from_slice(&rejoin.to_bytes());
        }
        bytes[VIEW_AT] = self.view.to_byte();
        bytes[UPDATE_AT..STATE_BYTES].copy_from_slice(&self.update.to_bytes());
        let mut words: [u32; Self::WORDS] = [0; Self::WORDS];
        for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(4)) {
            *word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        words[STATE_WORDS] = Crc32::checksum(&bytes);
        words
    }

    /// Reads [`to_words`](Self::to_words)' words. Words whose checksum does not match give the
    /// default: no page known and no rejoin, so the next wake joins slowly and draws the page.
    #[must_use]
    pub fn from_words(words: &[u32; Self::WORDS]) -> Self {
        let mut bytes: [u8; STATE_WORDS * 4] = [0; STATE_WORDS * 4];
        for (chunk, word) in bytes.chunks_exact_mut(4).zip(words) {
            chunk.copy_from_slice(&word.to_le_bytes());
        }
        if Crc32::checksum(&bytes) != words[STATE_WORDS] {
            return Self::default();
        }
        let mut wake: [u8; WakeState::BYTES] = [0; WakeState::BYTES];
        wake.copy_from_slice(&bytes[..WakeState::BYTES]);
        let mut rejoin: [u8; Rejoin::BYTES] = [0; Rejoin::BYTES];
        rejoin.copy_from_slice(&bytes[REJOIN_AT..VIEW_AT]);
        let mut update: [u8; FirmwareUpdate::BYTES] = [0; FirmwareUpdate::BYTES];
        update.copy_from_slice(&bytes[UPDATE_AT..STATE_BYTES]);
        Self {
            wake: WakeState::from_bytes(&wake),
            rejoin: if bytes[REJOIN_FLAG] == 1 {
                Rejoin::from_bytes(&rejoin)
            } else {
                None
            },
            view: PageView::from_byte(bytes[VIEW_AT]),
            update: FirmwareUpdate::from_bytes(&update),
        }
    }
}

#[cfg(test)]
mod test {
    use core::net::Ipv4Addr;

    use super::*;

    #[test]
    fn keeps_every_part_and_rejects_damage() {
        let memory = SleepMemory {
            wake: WakeState::default().with_etag(Some("\"0000abcd-z\"")),
            rejoin: Some(Rejoin {
                bssid: [1, 2, 3, 4, 5, 6],
                channel: 11,
                address: Ipv4Addr::new(10, 0, 0, 9),
                prefix_len: 16,
                gateway: None,
            }),
            view: PageView::Story,
            update: FirmwareUpdate::default().failed("0.3.0"),
        };
        let mut words: [u32; SleepMemory::WORDS] = memory.to_words();
        assert_eq!(SleepMemory::from_words(&words), memory);
        words[3] ^= 0x100;
        assert_eq!(SleepMemory::from_words(&words), SleepMemory::default());
    }

    #[test]
    fn buttons_redraw_or_step_the_view() {
        let memory = SleepMemory {
            wake: WakeState::default().with_etag(Some("\"0000abcd-z\"")),
            ..SleepMemory::default()
        };
        let refreshed: SleepMemory = memory.after_press(Button::Refresh);
        assert_eq!(
            (refreshed.wake.etag(), refreshed.view),
            (None, PageView::Today)
        );
        let next: SleepMemory = memory.after_press(Button::Next);
        assert_eq!(next.view, PageView::Week);
        assert_eq!(next.wake.etag(), Some("\"0000abcd-z\""));
        assert_eq!(next.after_press(Button::Previous).view, PageView::Today);
        assert_eq!(memory.after_press(Button::Previous).view, PageView::Season);
    }
}
