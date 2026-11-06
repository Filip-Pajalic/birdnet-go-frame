//! Reading the head of a request or response: the first line and the header fields.

use std::io::{self, Read};

/// Largest head accepted, in bytes.
pub(crate) const MAX_HEAD_BYTES: usize = 16 * 1024;

/// A parsed head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Head {
    /// The request or status line.
    pub(crate) first_line: String,
    /// Header fields in order, names as sent.
    pub(crate) fields: Vec<(String, String)>,
}

impl Head {
    /// Returns the value of the first field named `name`, ignoring case.
    pub(crate) fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(field, _)| field.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Reads from `stream` until the blank line that ends the head. Returns the head and any body
/// bytes read past it, or `None` when the stream ends before a head arrives.
pub(crate) fn read_head(stream: &mut impl Read) -> io::Result<Option<(Head, Vec<u8>)>> {
    let mut buffer: Vec<u8> = Vec::with_capacity(1024);
    let mut chunk: [u8; 1024] = [0; 1024];
    loop {
        if let Some(end) = find(&buffer, b"\r\n\r\n") {
            let head: Head = parse(&buffer[..end])?;
            return Ok(Some((head, buffer[end + 4..].to_vec())));
        }
        if buffer.len() > MAX_HEAD_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "head too large"));
        }
        let read: usize = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(None);
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
}

fn parse(bytes: &[u8]) -> io::Result<Head> {
    let text: &str = std::str::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "head is not UTF-8"))?;
    let mut lines = text.split("\r\n");
    let first_line: String = lines.next().unwrap_or_default().to_owned();
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "header without `:`"))?;
        fields.push((name.trim().to_owned(), value.trim().to_owned()));
    }
    Ok(Head { first_line, fields })
}

/// Returns where `needle` first starts in `haystack`.
pub(crate) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_a_head_and_keeps_the_rest() {
        let mut input: &[u8] = b"GET / HTTP/1.1\r\nHost: x\r\nIf-None-Match: \"1\"\r\n\r\nbody";
        let (head, rest) = read_head(&mut input).expect("readable").expect("complete");
        assert_eq!(head.first_line, "GET / HTTP/1.1");
        assert_eq!(head.field("if-none-match"), Some("\"1\""));
        assert_eq!(head.field("Accept"), None);
        assert_eq!(rest, b"body");
    }

    #[test]
    fn notices_truncation_and_garbage() {
        let mut short: &[u8] = b"GET / HTTP/1.1\r\n";
        assert!(read_head(&mut short).expect("readable").is_none());
        let mut garbage: &[u8] = b"GET / HTTP/1.1\r\nnonsense\r\n\r\n";
        assert!(read_head(&mut garbage).is_err());
    }
}
