//! The decisions of a frame that draws its own page, with no server between it and BirdNET-Go.
//!
//! The Raspberry Pi app (`apps/linux-panel`) and the standalone ESP32 firmware
//! (`apps/ee02-standalone`) both redraw only when the birds change: [`Fingerprint`] says
//! whether they have. The ESP32 also sends its own HTTP requests and sleeps between wakes with
//! nothing but a few words of RTC memory: [`birdnet_request`] and [`BirdnetAnswer`] are its side
//! of the conversation with BirdNET-Go, [`http_date`] gives it the time, and [`StandaloneMemory`]
//! remembers what is on the glass and plans the sleep. Everything here is plain logic, tested on
//! a laptop. `no_std` with `alloc`.
//!
//! ```
//! use frame_standalone::{BirdnetAnswer, StandaloneMemory};
//!
//! let response = b"HTTP/1.1 200 OK\r\nDate: Fri, 06 Nov 2026 07:00:00 GMT\r\n\r\n\
//!     [{\"scientific_name\":\"Parus major\",\"common_name\":\"Great Tit\",\"count\":3}]";
//! let answer = BirdnetAnswer::parse(response).expect("an HTTP response");
//! assert_eq!(answer.date, Some(1_793_948_400));
//! let species = answer.species().expect("a species list");
//!
//! let memory = StandaloneMemory::default();
//! let day: i64 = 1_793_948_400 / 86_400;
//! let wake = memory.after_species(day, &[&species], frame_artist::Layout::Collage, 15);
//! assert!(wake.show, "nothing was on the glass yet");
//! ```

#![no_std]

extern crate alloc;

mod birdnet_answer;
mod birdnet_request;
mod drawing_resolution;
mod fingerprint;
pub mod http_date;
mod standalone_memory;
mod standalone_settings;
mod view_fetch;

pub use birdnet_answer::{AnswerError, BirdnetAnswer};
pub use birdnet_request::birdnet_request;
pub use drawing_resolution::drawing_resolution;
pub use fingerprint::Fingerprint;
pub use standalone_memory::{StandaloneMemory, StandaloneWake};
pub use standalone_settings::{SettingsError, StandaloneSettings};
pub use view_fetch::{Summary, ViewFacts, ViewFetch};
