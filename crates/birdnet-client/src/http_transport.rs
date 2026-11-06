//! The one thing the client needs from a platform: sending an HTTP request.

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;

/// Sends HTTP requests for [`crate::BirdnetClient`].
///
/// A transport returns any HTTP status as a response; only failures to get a response at all
/// (DNS, connection, timeout) are errors.
pub trait HttpTransport {
    /// Why no response was received.
    type Error: core::error::Error;

    /// Sends `request`, following no redirects, and returns the response with its headers.
    ///
    /// # Errors
    ///
    /// Returns an error when no HTTP response was received.
    fn send(&mut self, request: &HttpRequest) -> Result<HttpResponse, Self::Error>;

    /// Fetches `url` with a plain GET.
    ///
    /// # Errors
    ///
    /// Returns an error when no HTTP response was received.
    fn get(&mut self, url: &str) -> Result<HttpResponse, Self::Error> {
        self.send(&HttpRequest::get(url))
    }
}
