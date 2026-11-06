//! Just enough plain HTTP/1.1 for a home network, on the standard library alone.
//!
//! [`get`] fetches a URL and [`send`] sends any request; [`Server`] answers GET and HEAD requests with a handler. Both close
//! the connection after one exchange, which keeps them small and is all a panel that wakes
//! every few minutes needs. There is no TLS: BirdNET-Go and the frame talk on the LAN. This
//! replaces `ureq` and `tiny_http` and the dozen crates they bring.
//!
//! ```no_run
//! use std::time::Duration;
//!
//! let response = lan_http::get("http://birdnet.local:8080/api/v2/health", Duration::from_secs(5))?;
//! assert_eq!(response.status, 200);
//! # Ok::<(), lan_http::HttpError>(())
//! ```

mod client;
mod head;
mod http_error;
mod request;
mod response;
mod server;
mod url;

pub use client::{ClientResponse, MAX_BODY_BYTES, get, send};
pub use http_error::HttpError;
pub use request::Request;
pub use response::Response;
pub use server::Server;
