//! The ureq-backed transport.

use std::time::Duration;

use birdnet_client::{HttpResponse, HttpTransport};
use ureq::Agent;

/// The largest body accepted from BirdNET-Go. A summary for a busy station is well under this.
const MAX_BODY_BYTES: u64 = 16 * 1024 * 1024;

/// Performs blocking HTTP GETs with a global timeout.
#[derive(Clone, Debug)]
pub struct UreqTransport {
    agent: Agent,
}

impl UreqTransport {
    /// Creates a transport whose requests give up after `timeout`.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(timeout))
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .into();
        Self { agent }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new(Duration::from_secs(10))
    }
}

impl HttpTransport for UreqTransport {
    type Error = ureq::Error;

    fn get(&mut self, url: &str) -> Result<HttpResponse, Self::Error> {
        let mut response = self.agent.get(url).call()?;
        let status: u16 = response.status().as_u16();
        let body: Vec<u8> = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_vec()?;
        Ok(HttpResponse { status, body })
    }
}
