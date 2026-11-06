//! The words kept in RTC fast memory through deep sleep.

use frame_standalone::StandaloneMemory;
use portable_atomic::{AtomicU32, Ordering};

/// The [`StandaloneMemory`] words. esp-hal zeroes them on the first boot and leaves them alone
/// after a deep sleep or a reset; atomics make them safe to touch without `unsafe`.
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static WORDS: [AtomicU32; StandaloneMemory::WORDS] =
    [const { AtomicU32::new(0) }; StandaloneMemory::WORDS];

/// Reads what the last wake left. A first boot or damaged memory reads as the default.
pub fn load() -> StandaloneMemory {
    let mut words: [u32; StandaloneMemory::WORDS] = [0; StandaloneMemory::WORDS];
    for (word, kept) in words.iter_mut().zip(&WORDS) {
        *word = kept.load(Ordering::Relaxed);
    }
    StandaloneMemory::from_words(&words)
}

/// Keeps `memory` for the next wake.
pub fn store(memory: &StandaloneMemory) {
    for (kept, word) in WORDS.iter().zip(memory.to_words()) {
        kept.store(word, Ordering::Relaxed);
    }
}
