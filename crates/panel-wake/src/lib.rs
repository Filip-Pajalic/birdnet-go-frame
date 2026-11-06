//! What a panel does each time it wakes, apart from the radio and the glass.
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
//! ```
//!
//! The firmware connects, sends the request, reads the head and body, and then shows the page
//! or not and sleeps for as long as this crate says.

#![no_std]

extern crate alloc;

mod page_request;
mod response_head;
mod wake;
mod wake_error;
mod wake_state;

pub use page_request::page_request;
pub use response_head::ResponseHead;
pub use wake::{SleepLimits, Wake};
pub use wake_error::WakeError;
pub use wake_state::WakeState;
