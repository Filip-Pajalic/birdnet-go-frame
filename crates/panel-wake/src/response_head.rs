//! The part of the server's answer the panel acts on.

use alloc::string::{String, ToString};

/// A response's status line and the headers a panel needs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResponseHead {
    /// The status code: 200, 304, 503, …
    pub status: u16,
    /// The `ETag`, quotes included, to send back next time.
    pub etag: Option<String>,
    /// Whether the body is a zlib stream (`Content-Encoding: deflate`).
    pub deflate: bool,
    /// The body's length on the wire, from `Content-Length`.
    pub content_length: Option<usize>,
    /// Seconds to sleep before asking again, from `Cache-Control: max-age`.
    pub max_age: Option<u32>,
}

impl ResponseHead {
    /// Reads the head: everything before the blank line, which may be included or not.
    ///
    /// Returns `None` when the status line is not HTTP.
    #[must_use]
    pub fn parse(head: &[u8]) -> Option<Self> {
        let text: &str = core::str::from_utf8(head).ok()?;
        let mut lines = text.split("\r\n");
        let status_line: &str = lines.next()?;
        let mut parts = status_line.split(' ');
        if !parts.next()?.starts_with("HTTP/1.") {
            return None;
        }
        let status: u16 = parts.next()?.parse().ok()?;
        let mut head = Self {
            status,
            ..Self::default()
        };
        for line in lines.take_while(|line| !line.is_empty()) {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            let value: &str = value.trim();
            if name.eq_ignore_ascii_case("etag") {
                head.etag = Some(value.to_string());
            } else if name.eq_ignore_ascii_case("content-encoding") {
                head.deflate = value.eq_ignore_ascii_case("deflate");
            } else if name.eq_ignore_ascii_case("content-length") {
                head.content_length = value.parse().ok();
            } else if name.eq_ignore_ascii_case("cache-control") {
                head.max_age = value
                    .split(',')
                    .filter_map(|directive| directive.trim().strip_prefix("max-age="))
                    .find_map(|seconds| seconds.parse().ok());
            }
        }
        Some(head)
    }

    /// Splits a response read so far into its head and the body bytes after it.
    ///
    /// Returns `None` until the blank line that ends the head has arrived, or when the status
    /// line is not HTTP.
    ///
    /// ```
    /// use panel_wake::ResponseHead;
    ///
    /// let (head, body) =
    ///     ResponseHead::split(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\nab").expect("a head");
    /// assert_eq!((head.status, body), (200, &b"ab"[..]));
    /// assert!(!head.is_complete(body));
    /// assert!(ResponseHead::split(b"HTTP/1.1 200 OK\r\nContent-Le").is_none());
    /// ```
    #[must_use]
    pub fn split(response: &[u8]) -> Option<(Self, &[u8])> {
        let end: usize = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")?;
        let head = Self::parse(&response[..end])?;
        Some((head, &response[end + 4..]))
    }

    /// Whether `body` is all of this response's body, by its `Content-Length`. Without one, the
    /// body ends only when the server closes the connection, so this says `false`.
    #[must_use]
    pub fn is_complete(&self, body: &[u8]) -> bool {
        self.content_length
            .is_some_and(|length| body.len() >= length)
    }

    /// The CRC-32 the server put in the `ETag`: the page's, before compression.
    #[must_use]
    pub fn page_crc(&self) -> Option<u32> {
        let tag: &str = self.etag.as_deref()?.trim_matches('"');
        let hex: &str = tag.strip_suffix("-z").unwrap_or(tag);
        u32::from_str_radix(hex, 16).ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_what_the_server_sends() {
        let head = ResponseHead::parse(
            b"HTTP/1.1 200 OK\r\nContent-Length: 216303\r\nContent-Type: application/octet-stream\r\n\
              Cache-Control: max-age=900\r\nETag: \"b35332bb-z\"\r\nVary: Accept-Encoding\r\n\
              Content-Encoding: deflate\r\n\r\n",
        )
        .expect("HTTP");
        assert_eq!(head.status, 200);
        assert!(head.deflate);
        assert_eq!(head.content_length, Some(216_303));
        assert_eq!(head.max_age, Some(900));
        assert_eq!(head.page_crc(), Some(0xb353_32bb));

        let unchanged =
            ResponseHead::parse(b"HTTP/1.1 304 Not Modified\r\nCache-Control: no-cache")
                .expect("HTTP");
        assert_eq!((unchanged.status, unchanged.max_age), (304, None));
        assert!(ResponseHead::parse(b"SSH-2.0-OpenSSH").is_none());
    }

    #[test]
    fn splits_off_the_body_once_the_head_is_in() {
        let response: &[u8] = b"HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n";
        let (head, body) = ResponseHead::split(response).expect("a head");
        assert_eq!((head.status, body.len()), (304, 0));
        assert!(head.is_complete(body));
        for cut in 0..response.len() - 1 {
            assert!(ResponseHead::split(&response[..cut]).is_none(), "{cut}");
        }
        let (unsized_head, _) = ResponseHead::split(b"HTTP/1.1 200 OK\r\n\r\nabc").expect("a head");
        assert!(!unsized_head.is_complete(b"abc"));
    }
}
