//! An incoming request.

use crate::head::Head;

/// A request the [`crate::Server`] received.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    method: String,
    target: String,
    head: Head,
}

impl Request {
    pub(crate) fn from_head(head: Head) -> Option<Self> {
        let mut parts = head.first_line.split(' ');
        let (Some(method), Some(target), Some(version)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        if !version.starts_with("HTTP/1.") || !target.starts_with('/') {
            return None;
        }
        Some(Self {
            method: method.to_owned(),
            target: target.to_owned(),
            head,
        })
    }

    /// Returns the method, e.g. `GET`.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the path and query, e.g. `/frame.e6?x=1`.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the path without the query.
    #[must_use]
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("/")
    }

    /// Returns the value of the header named `name`, ignoring case.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.head.field(name)
    }
}
