//! The words kept in RTC fast memory through deep sleep.

use panel_wake::SleepMemory;
use portable_atomic::{AtomicU32, Ordering};

/// The [`SleepMemory`] words. esp-hal zeroes them on the first boot and leaves them alone after
/// a deep sleep or a reset; atomics make them safe to touch without `unsafe`.
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static WORDS: [AtomicU32; SleepMemory::WORDS] = [const { AtomicU32::new(0) }; SleepMemory::WORDS];

/// Reads what the last wake left. A first boot or damaged memory reads as the default.
pub fn load() -> SleepMemory {
    let mut words: [u32; SleepMemory::WORDS] = [0; SleepMemory::WORDS];
    for (word, kept) in words.iter_mut().zip(&WORDS) {
        *word = kept.load(Ordering::Relaxed);
    }
    SleepMemory::from_words(&words)
}

/// Keeps `memory` for the next wake.
pub fn store(memory: &SleepMemory) {
    for (kept, word) in WORDS.iter().zip(memory.to_words()) {
        kept.store(word, Ordering::Relaxed);
    }
}
