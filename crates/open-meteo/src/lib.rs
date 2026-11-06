//! Today's weather from [Open-Meteo](https://open-meteo.com/), which needs no key or account.
//!
//! [`OpenMeteo`] is generic over `birdnet-client`'s [`birdnet_client::HttpTransport`], so it
//! runs wherever the BirdNET-Go client does. It asks for plain HTTP, which Open-Meteo serves,
//! because the server's transport speaks no TLS. `no_std` with `alloc`.
//!
//! ```
//! use birdnet_mock::MockTransport;
//! use open_meteo::{OpenMeteo, Sky};
//!
//! let now: i64 = 1_793_955_600; // 2026-11-06T09:00:00Z
//! let mut weather = OpenMeteo::new("http://mock", MockTransport::new(move || now));
//! let today = weather.today(59.33, 18.07).expect("the mock answers like Open-Meteo");
//! assert_eq!(today.sunrise.len(), 5);
//! assert_ne!(today.sky, Sky::Unknown);
//! ```

#![no_std]

extern crate alloc;

mod open_meteo;
mod sky;
mod weather;
mod weather_error;

pub use open_meteo::OpenMeteo;
pub use sky::Sky;
pub use weather::Weather;
pub use weather_error::WeatherError;
