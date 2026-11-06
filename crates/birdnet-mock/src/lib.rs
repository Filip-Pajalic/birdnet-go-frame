//! A simulated BirdNET-Go for tests, demos and development without a microphone.
//!
//! [`MockBirdnet`] answers the API paths the frame reads with realistic JSON: a garden's worth
//! of European species whose detections build up through the day, with a dawn chorus. The
//! output depends only on the time it is asked for, so tests are deterministic.
//!
//! [`MockTransport`] plugs it into [`birdnet_client::BirdnetClient`] in-process; the
//! `fake-birdnet` app serves it over HTTP so anything that speaks to BirdNET-Go can use it.
//! `no_std` with `alloc`.
//!
//! ```
//! use birdnet_client::{BirdnetClient, DateWindow};
//! use birdnet_mock::MockTransport;
//!
//! let now: i64 = 1_793_955_600; // 2026-11-06T09:00:00Z
//! let mut client = BirdnetClient::new("mock", MockTransport::new(move || now));
//! let species = client.species_summary(DateWindow::hours_back(now, 0, 24)).expect("mock answers");
//! assert!(!species.is_empty());
//! ```

#![no_std]

extern crate alloc;

mod garden;
mod mock_birdnet;
mod mock_transport;

pub use mock_birdnet::MockBirdnet;
pub use mock_transport::MockTransport;
