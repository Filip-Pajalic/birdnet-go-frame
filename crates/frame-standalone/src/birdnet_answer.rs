//! Reading BirdNET-Go's answer.

use alloc::vec::Vec;
use core::fmt;

use birdnet_model::{ParseError, SpeciesSummary};

use crate::http_date;

/// An HTTP response from BirdNET-Go: the parts a standalone frame needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BirdnetAnswer<'a> {
    /// The status code.
    pub status: u16,
    /// The unix time in the `Date` header, if it had one the frame can read.
    pub date: Option<i64>,
    /// The body, cut to `Content-Length` when there is one.
    pub body: &'a [u8],
}

/// Why BirdNET-Go's answer gave no birds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum AnswerError {
    /// The bytes are not an HTTP response.
    NotHttp,
    /// The body ended before its `Content-Length`.
    CutShort,
    /// BirdNET-Go asked for a login: its private mode is on.
    Unauthorized,
    /// BirdNET-Go answered with another status.
    Status(u16),
    /// The body is not a species list.
    Parse(ParseError),
}

impl fmt::Display for AnswerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotHttp => formatter.write_str("BirdNET-Go's answer is not HTTP"),
            Self::CutShort => formatter.write_str("BirdNET-Go's answer was cut short"),
            Self::Unauthorized => formatter
                .write_str("BirdNET-Go wants a login; turn its private mode off or use the server"),
            Self::Status(status) => write!(formatter, "BirdNET-Go answered {status}"),
            Self::Parse(error) => write!(formatter, "BirdNET-Go's species list: {error}"),
        }
    }
}

impl core::error::Error for AnswerError {}

impl<'a> BirdnetAnswer<'a> {
    /// Reads the head of `response`, a whole HTTP response as it came off the wire.
    ///
    /// # Errors
    ///
    /// Returns [`AnswerError::NotHttp`] when there is no status line and blank line, or
    /// [`AnswerError::CutShort`] when the body is shorter than its `Content-Length`.
    pub fn parse(response: &'a [u8]) -> Result<Self, AnswerError> {
        let split: usize = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or(AnswerError::NotHttp)?;
        let head: &str =
            core::str::from_utf8(&response[..split]).map_err(|_| AnswerError::NotHttp)?;
        let body: &[u8] = &response[split + 4..];
        let mut lines = head.split("\r\n");
        let status_line: &str = lines.next().ok_or(AnswerError::NotHttp)?;
        let mut words = status_line.split(' ');
        if !words
            .next()
            .is_some_and(|version| version.starts_with("HTTP/"))
        {
            return Err(AnswerError::NotHttp);
        }
        let status: u16 = words
            .next()
            .and_then(|code| code.parse().ok())
            .ok_or(AnswerError::NotHttp)?;
        let mut date: Option<i64> = None;
        let mut length: Option<usize> = None;
        for line in lines {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            if name.trim().eq_ignore_ascii_case("date") {
                date = http_date::parse(value);
            } else if name.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().ok();
            }
        }
        let body: &[u8] = match length {
            Some(length) => body.get(..length).ok_or(AnswerError::CutShort)?,
            None => body,
        };
        Ok(Self { status, date, body })
    }

    /// Returns the birds in a species summary answer, most heard first.
    ///
    /// # Errors
    ///
    /// Returns [`AnswerError::Unauthorized`] for a 401 or 403, [`AnswerError::Status`] for any
    /// other status outside 2xx, or [`AnswerError::Parse`] when the body is not a species list.
    pub fn species(&self) -> Result<Vec<SpeciesSummary>, AnswerError> {
        match self.status {
            200..=299 => SpeciesSummary::parse_list(self.body).map_err(AnswerError::Parse),
            401 | 403 => Err(AnswerError::Unauthorized),
            status => Err(AnswerError::Status(status)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_status_date_and_body() {
        let response: &[u8] = b"HTTP/1.0 200 OK\r\ncontent-length: 2\r\n\
            date: Fri, 06 Nov 2026 07:00:00 GMT\r\n\r\n[]trailing";
        let answer: BirdnetAnswer<'_> = BirdnetAnswer::parse(response).expect("HTTP");
        assert_eq!(answer.status, 200);
        assert_eq!(answer.date, Some(1_793_948_400));
        assert_eq!(answer.body, b"[]");
        assert_eq!(answer.species(), Ok(Vec::new()));
    }

    #[test]
    fn tells_what_went_wrong() {
        assert_eq!(BirdnetAnswer::parse(b"garbage"), Err(AnswerError::NotHttp));
        assert_eq!(
            BirdnetAnswer::parse(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\n\r\n[]"),
            Err(AnswerError::CutShort)
        );
        let locked = BirdnetAnswer::parse(b"HTTP/1.1 401 Unauthorized\r\n\r\n").expect("HTTP");
        assert_eq!(locked.species(), Err(AnswerError::Unauthorized));
        let missing = BirdnetAnswer::parse(b"HTTP/1.1 404 Not Found\r\n\r\n").expect("HTTP");
        assert_eq!(missing.species(), Err(AnswerError::Status(404)));
        assert_eq!(missing.date, None);
    }
}
