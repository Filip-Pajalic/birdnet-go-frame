//! A client for the parts of BirdNET-Go's `/api/v2` the frame reads.
//!
//! The client is generic over [`HttpTransport`], so the same code runs on Linux (with
//! `transport-lan`) and on the ESP32. It is `no_std` with `alloc`. When BirdNET-Go's private
//! mode is on, [`BirdnetClient::with_login`] logs in with its password and keeps the session.
//!
//! ```
//! use birdnet_client::{BirdnetClient, DateWindow, HttpRequest, HttpResponse, HttpTransport};
//!
//! struct Canned;
//!
//! impl HttpTransport for Canned {
//!     type Error = core::convert::Infallible;
//!
//!     fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, Self::Error> {
//!         assert!(request.url.ends_with("/api/v2/analytics/species/summary?start_date=2026-11-06&end_date=2026-11-06"));
//!         let body = br#"[{"scientific_name":"Parus major","common_name":"Great Tit","count":3}]"#;
//!         Ok(HttpResponse::new(200, body.to_vec()))
//!     }
//! }
//!
//! let mut client = BirdnetClient::new("http://birdnet.local:8080/", Canned);
//! // 2026-11-06T07:00:00Z, looking back 6 hours in UTC.
//! let window = DateWindow::hours_back(1_793_948_400, 0, 6);
//! let species = client.species_summary(window).expect("canned response parses");
//! assert_eq!(species[0].common_name, "Great Tit");
//! ```

#![no_std]

extern crate alloc;

mod birdnet_client;
mod civil_date;
mod client_error;
mod date_window;
mod http_request;
mod http_response;
mod http_transport;

pub use birdnet_client::BirdnetClient;
pub use civil_date::CivilDate;
pub use client_error::ClientError;
pub use date_window::DateWindow;
pub use http_request::{HttpRequest, Method};
pub use http_response::HttpResponse;
pub use http_transport::HttpTransport;
