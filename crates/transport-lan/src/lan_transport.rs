//! The lan-http transport.

use std::time::Duration;

use birdnet_client::{HttpResponse, HttpTransport};
use lan_http::HttpError;

/// Performs blocking plain-HTTP GETs with a timeout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanTransport {
    timeout: Duration,
}

impl LanTransport {
    /// Creates a transport whose connects, reads and writes each give up after `timeout`.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for LanTransport {
    fn default() -> Self {
        Self::new(Duration::from_secs(10))
    }
}

impl HttpTransport for LanTransport {
    type Error = HttpError;

    fn get(&mut self, url: &str) -> Result<HttpResponse, Self::Error> {
        let response = lan_http::get(url, self.timeout)?;
        Ok(HttpResponse {
            status: response.status,
            body: response.body,
        })
    }
}
