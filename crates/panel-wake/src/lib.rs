//! What a panel does each time it wakes, apart from driving the radio and the glass.
//!
//! `no_std` with `alloc`, so the ESP32 firmware and a test on a laptop run the same decisions:
//!
//! ```text
//! WakeState (kept in RTC memory through deep sleep)
//!   │  page_request()                 GET /panel, If-None-Match, Accept-Encoding: deflate
//!   ▼
//! ResponseHead::parse()              status, ETag, encoding, Cache-Control: max-age
//!   │  WakeState::after_response()   304 keep, 200 unpack and check, else back off
//!   ▼
//! Wake { show, sleep_seconds, state }
//!   │  SleepMemory::to_words()        with the Wi-Fi Rejoin record, checksummed
//!   ▼
//! RTC memory, until the next wake
//! ```
//!
//! The firmware connects, sends the request, reads the head and body, and then shows the page
//! or not and sleeps for as long as this crate says.

#![no_std]

extern crate alloc;

mod frame_address;
mod page_request;
mod rejoin;
mod response_head;
mod sleep_memory;
mod wake;
mod wake_error;
mod wake_state;

pub use frame_address::{FrameAddress, ParseFrameAddressError};
pub use page_request::page_request;
pub use rejoin::Rejoin;
pub use response_head::ResponseHead;
pub use sleep_memory::SleepMemory;
pub use wake::{SleepLimits, Wake};
pub use wake_error::WakeError;
pub use wake_state::WakeState;
