//! A request for a transport to send.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The HTTP methods the client uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Reads.
    Get,
    /// Logs in.
    Post,
}

impl Method {
    /// The method as it goes on the wire.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

/// One HTTP request: method, URL, extra headers and body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    /// The method.
    pub method: Method,
    /// The full URL, e.g. `http://192.168.1.10:8080/api/v2/...`.
    pub url: String,
    /// Headers besides `Host`, `Content-Length` and `Connection`, which the transport adds.
    pub headers: Vec<(String, String)>,
    /// The body; empty for a GET.
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// A GET of `url`.
    #[must_use]
    pub fn get(url: &str) -> Self {
        Self {
            method: Method::Get,
            url: url.to_string(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// A POST of the JSON `body` to `url`.
    #[must_use]
    pub fn post_json(url: &str, body: Vec<u8>) -> Self {
        Self {
            method: Method::Post,
            url: url.to_string(),
            headers: Vec::new(),
            body,
        }
        .with_header("Content-Type", "application/json")
    }

    /// Returns the request with one more header.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    /// The value of the header `name`, ignoring case.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(field, _)| field.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}
