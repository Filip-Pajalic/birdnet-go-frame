//! The request a standalone frame sends BirdNET-Go.

use alloc::format;
use alloc::string::String;

/// Returns an HTTP GET of `path` (with its query) for BirdNET-Go at `host`, e.g.
/// `192.168.1.10:8080`.
///
/// It is HTTP/1.0 so the answer is never chunked, and asks the server to close the connection
/// after it, which is how the frame knows the answer has ended when it has no
/// `Content-Length`.
#[must_use]
pub fn birdnet_request(host: &str, path: &str) -> String {
    format!(
        "GET {path} HTTP/1.0\r\nHost: {host}\r\nAccept: application/json\r\n\
         User-Agent: birdnet-go-frame\r\nConnection: close\r\n\r\n"
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn asks_for_the_path_and_a_closed_connection() {
        let request: String = birdnet_request("192.168.1.10:8080", "/api/v2/health");
        assert!(request.starts_with("GET /api/v2/health HTTP/1.0\r\nHost: 192.168.1.10:8080\r\n"));
        assert!(request.ends_with("Connection: close\r\n\r\n"));
    }
}
