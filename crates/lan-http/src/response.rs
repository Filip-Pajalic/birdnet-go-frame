//! A response for the server to send.

/// What the [`crate::Server`] sends back. `Content-Length` and `Connection: close` are added
/// when it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
}

impl Response {
    /// Creates a response with `status` and `body`.
    #[must_use]
    pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.into(),
        }
    }

    /// Creates a plain-text response.
    #[must_use]
    pub fn text(status: u16, body: &str) -> Self {
        Self::new(status, body).with_header("Content-Type", "text/plain; charset=utf-8")
    }

    /// Adds a header.
    #[must_use]
    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }

    /// Returns the status code.
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Returns the body.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Returns the bytes to send: the head, and the body unless `head_only`.
    pub(crate) fn to_bytes(&self, head_only: bool) -> Vec<u8> {
        let mut head: String = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.status,
            reason(self.status),
            self.body.len()
        );
        for (name, value) in &self.headers {
            head.push_str(name);
            head.push_str(": ");
            // A value never spans lines; drop anything that would split the head.
            head.extend(
                value
                    .chars()
                    .filter(|letter| *letter != '\r' && *letter != '\n'),
            );
            head.push_str("\r\n");
        }
        head.push_str("\r\n");
        let mut bytes: Vec<u8> = head.into_bytes();
        if !head_only {
            bytes.extend_from_slice(&self.body);
        }
        bytes
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Status",
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn writes_head_and_body() {
        let response = Response::text(200, "ok").with_header("ETag", "\"1\"\r\nX: injected");
        let bytes = String::from_utf8(response.to_bytes(false)).expect("ASCII");
        assert_eq!(
            bytes,
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\
             Content-Type: text/plain; charset=utf-8\r\nETag: \"1\"X: injected\r\n\r\nok"
        );
        assert!(
            !String::from_utf8(response.to_bytes(true))
                .expect("ASCII")
                .ends_with("ok")
        );
    }
}
