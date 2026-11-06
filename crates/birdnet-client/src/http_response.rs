//! A finished HTTP response.

use alloc::vec::Vec;

/// A finished HTTP response: its status and whole body.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body.
    pub body: Vec<u8>,
}
