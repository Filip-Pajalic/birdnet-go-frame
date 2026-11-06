//! A finished HTTP response.

use alloc::string::String;
use alloc::vec::Vec;

/// A finished HTTP response: its status, headers and whole body.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body.
    pub body: Vec<u8>,
    /// Header names and values, in order; names repeat, as `Set-Cookie` does.
    pub headers: Vec<(String, String)>,
}

impl HttpResponse {
    /// A response with `status` and `body` and no headers.
    #[must_use]
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            body,
            headers: Vec::new(),
        }
    }

    /// Every value of the header `name`, ignoring case.
    pub fn header_values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.headers
            .iter()
            .filter(move |(field, _)| field.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}
