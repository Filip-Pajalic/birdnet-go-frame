//! The lan-http transport.

use std::time::Duration;

use birdnet_client::{HttpRequest, HttpResponse, HttpTransport};
use lan_http::HttpError;

/// Sends blocking plain-HTTP requests with a timeout.
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

    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, Self::Error> {
        let headers: Vec<(&str, &str)> = request
            .headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let response = lan_http::send(
            request.method.as_str(),
            &request.url,
            &headers,
            &request.body,
            self.timeout,
        )?;
        Ok(HttpResponse {
            status: response.status,
            body: response.body,
            headers: response.headers,
        })
    }
}
