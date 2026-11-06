//! The one thing the client needs from a platform: an HTTP GET.

use crate::http_response::HttpResponse;

/// Performs HTTP GET requests for [`crate::BirdnetClient`].
///
/// A transport returns any HTTP status as a response; only failures to get a response at all
/// (DNS, connection, timeout) are errors.
pub trait HttpTransport {
    /// Why no response was received.
    type Error: core::error::Error;

    /// Fetches `url`, following no redirects.
    ///
    /// # Errors
    ///
    /// Returns an error when no HTTP response was received.
    fn get(&mut self, url: &str) -> Result<HttpResponse, Self::Error>;
}
