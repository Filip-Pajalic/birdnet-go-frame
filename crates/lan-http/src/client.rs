//! A blocking GET client.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::head::{self, Head};
use crate::http_error::HttpError;
use crate::url::Url;

/// The largest body [`get`] accepts. A summary from a busy BirdNET-Go is well under this.
pub const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// A response to [`get`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientResponse {
    /// Status code, e.g. 200. Redirects are not followed.
    pub status: u16,
    /// The body, with any chunked transfer encoding removed.
    pub body: Vec<u8>,
}

/// Fetches `url` (`http://host[:port]/path`) with one GET. Connecting, and each read and write,
/// give up after `timeout`.
///
/// # Errors
///
/// Returns [`HttpError`] when the URL is not plain `http://`, the server cannot be reached, or
/// the response is not readable HTTP/1.x.
pub fn get(url: &str, timeout: Duration) -> Result<ClientResponse, HttpError> {
    let url: Url<'_> = Url::parse(url)?;
    let mut stream: TcpStream = connect(&url, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let request: String = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: birdnet-go-frame/{}\r\nAccept: */*\r\n\
         Connection: close\r\n\r\n",
        url.target,
        url.authority,
        env!("CARGO_PKG_VERSION"),
    );
    stream.write_all(request.as_bytes())?;
    let (head, mut body) =
        head::read_head(&mut stream)?.ok_or(HttpError::Malformed("connection closed early"))?;
    let status: u16 = status(&head)?;
    let mut chunk: [u8; 16 * 1024] = [0; 16 * 1024];
    loop {
        let read: usize = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        if body.len() + read > MAX_BODY_BYTES + 64 * 1024 {
            return Err(HttpError::TooLarge);
        }
        body.extend_from_slice(&chunk[..read]);
    }
    let chunked: bool = head
        .field("Transfer-Encoding")
        .is_some_and(|coding| coding.eq_ignore_ascii_case("chunked"));
    let body: Vec<u8> = if chunked {
        dechunk(&body)?
    } else if let Some(length) = head.field("Content-Length") {
        let length: usize = length
            .parse()
            .map_err(|_| HttpError::Malformed("bad Content-Length"))?;
        if body.len() < length {
            return Err(HttpError::Malformed("body shorter than Content-Length"));
        }
        body.truncate(length);
        body
    } else {
        body
    };
    if body.len() > MAX_BODY_BYTES {
        return Err(HttpError::TooLarge);
    }
    Ok(ClientResponse { status, body })
}

fn connect(url: &Url<'_>, timeout: Duration) -> Result<TcpStream, HttpError> {
    let mut last_error: Option<std::io::Error> = None;
    for address in (url.host, url.port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, timeout) {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.map_or(HttpError::BadUrl("host has no address"), HttpError::Io))
}

fn status(head: &Head) -> Result<u16, HttpError> {
    let mut parts = head.first_line.split(' ');
    match (parts.next(), parts.next()) {
        (Some(version), Some(code)) if version.starts_with("HTTP/1.") => code
            .parse()
            .map_err(|_| HttpError::Malformed("bad status code")),
        _ => Err(HttpError::Malformed("not an HTTP/1.x status line")),
    }
}

/// Removes chunked transfer encoding.
fn dechunk(mut data: &[u8]) -> Result<Vec<u8>, HttpError> {
    let mut body: Vec<u8> = Vec::with_capacity(data.len());
    loop {
        let line_end: usize =
            head::find(data, b"\r\n").ok_or(HttpError::Malformed("unterminated chunk size"))?;
        let size_text: &str = std::str::from_utf8(&data[..line_end])
            .map_err(|_| HttpError::Malformed("bad chunk size"))?;
        let size_text: &str = size_text.split(';').next().unwrap_or_default().trim();
        let size: usize = usize::from_str_radix(size_text, 16)
            .map_err(|_| HttpError::Malformed("bad chunk size"))?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Ok(body);
        }
        if body.len() + size > MAX_BODY_BYTES {
            return Err(HttpError::TooLarge);
        }
        let chunk: &[u8] = data
            .get(..size)
            .ok_or(HttpError::Malformed("chunk shorter than its size"))?;
        body.extend_from_slice(chunk);
        data = data
            .get(size..)
            .and_then(|rest| rest.strip_prefix(b"\r\n"))
            .ok_or(HttpError::Malformed("chunk not followed by CRLF"))?;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn removes_chunking() {
        let body =
            dechunk(b"4\r\nWiki\r\n6;ext=1\r\npedia \r\nE\r\nin \r\n\r\nchunks.\r\n0\r\n\r\n")
                .expect("valid");
        assert_eq!(body, b"Wikipedia in \r\n\r\nchunks.");
        assert!(dechunk(b"5\r\nabc").is_err());
        assert!(dechunk(b"zz\r\n").is_err());
    }

    #[test]
    fn reads_status_lines() {
        let head = |line: &str| Head {
            first_line: line.to_owned(),
            fields: Vec::new(),
        };
        assert_eq!(status(&head("HTTP/1.1 304 Not Modified")).ok(), Some(304));
        assert!(status(&head("SSH-2.0-OpenSSH")).is_err());
    }
}
