//! The HTTP the setup page needs: reading a browser's request, writing the answer.

use alloc::format;
use alloc::vec::Vec;

/// The setup page's own address, which is the hotspot's gateway.
pub const PAGE_URL: &str = "http://192.168.4.1/";

/// The request methods the setup page tells apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    /// `GET`: show the page.
    Get,
    /// `POST`: save the form.
    Post,
    /// Anything else.
    Other,
}

/// A browser's request to the setup page, once its head has arrived.
///
/// ```
/// use panel_setup::{Method, SetupRequest};
///
/// let (request, body) =
///     SetupRequest::split(b"POST /save HTTP/1.1\r\nContent-Length: 6\r\n\r\nssid=a").expect("a head");
/// assert_eq!((request.method, request.path), (Method::Post, "/save"));
/// assert!(request.is_complete(body));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetupRequest<'a> {
    /// The method.
    pub method: Method,
    /// The path, without the query.
    pub path: &'a str,
    /// The body's length, from `Content-Length`; none means no body.
    pub content_length: usize,
}

impl<'a> SetupRequest<'a> {
    /// Splits a request read so far into its head and the body bytes after it.
    ///
    /// Returns `None` until the blank line that ends the head has arrived, or when the request
    /// line is not HTTP.
    #[must_use]
    pub fn split(request: &'a [u8]) -> Option<(Self, &'a [u8])> {
        let end: usize = request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")?;
        let head: &str = core::str::from_utf8(&request[..end]).ok()?;
        let mut lines = head.split("\r\n");
        let mut parts = lines.next()?.split(' ');
        let method: Method = match parts.next()? {
            "GET" | "HEAD" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Other,
        };
        let target: &str = parts.next()?;
        if !parts.next()?.starts_with("HTTP/1.") {
            return None;
        }
        let path: &str = target.split('?').next().unwrap_or(target);
        let content_length: usize = lines
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
            .and_then(|(_, value)| value.trim().parse().ok())
            .unwrap_or(0);
        Some((
            Self {
                method,
                path,
                content_length,
            },
            &request[end + 4..],
        ))
    }

    /// Whether `body` is all of the request's body.
    #[must_use]
    pub fn is_complete(&self, body: &[u8]) -> bool {
        body.len() >= self.content_length
    }
}

/// A whole HTTP/1.1 response that closes the connection.
#[must_use]
pub fn http_response(status: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    let mut response: Vec<u8> = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

/// The answer to any address but the page's: a redirect to the page, which is also what makes a
/// phone show its "sign in to network" prompt.
#[must_use]
pub fn redirect_to_page() -> Vec<u8> {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {PAGE_URL}\r\nContent-Length: 0\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n"
    )
    .into_bytes()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_requests_and_waits_for_the_body() {
        let request: &[u8] =
            b"POST /save?x=1 HTTP/1.1\r\nHost: 192.168.4.1\r\ncontent-length: 10\r\n\r\nssid=";
        let (head, body) = SetupRequest::split(request).expect("a head");
        assert_eq!(
            (head.method, head.path, head.content_length),
            (Method::Post, "/save", 10)
        );
        assert!(!head.is_complete(body));
        let (get, _) = SetupRequest::split(b"GET /generate_204 HTTP/1.1\r\n\r\n").expect("a head");
        assert_eq!(
            (get.method, get.path, get.content_length),
            (Method::Get, "/generate_204", 0)
        );
        assert!(SetupRequest::split(b"GET / HTTP/1.1\r\nHost").is_none());
        assert!(SetupRequest::split(b"SSH-2.0\r\n\r\n").is_none());
        let (other, _) = SetupRequest::split(b"PUT / HTTP/1.0\r\n\r\n").expect("a head");
        assert_eq!(other.method, Method::Other);
    }

    #[test]
    fn writes_whole_answers() {
        let response: Vec<u8> = http_response("200 OK", "text/plain", b"hi");
        assert!(
            response.starts_with(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n"
            )
        );
        assert!(response.ends_with(b"\r\n\r\nhi"));
        let redirect: Vec<u8> = redirect_to_page();
        assert!(redirect.starts_with(b"HTTP/1.1 302 Found\r\nLocation: http://192.168.4.1/\r\n"));
    }
}
