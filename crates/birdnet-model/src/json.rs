//! A small JSON reader, enough for BirdNET-Go's responses.
//!
//! Replaces serde and serde_json (and the proc-macro crates they bring) with about two hundred
//! lines. It builds a tree, which is fine for responses of a few hundred kilobytes.

use alloc::string::String;
use alloc::vec::Vec;

use crate::parse_error::ParseError;

/// Deepest nesting accepted, so a hostile body cannot exhaust the stack.
const MAX_DEPTH: usize = 64;

/// A JSON value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    /// Members in document order.
    Object(Vec<(String, Json)>),
}

impl Json {
    /// Parses a whole document.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut reader = Reader { bytes, at: 0 };
        let value: Self = reader.value(0)?;
        reader.skip_space();
        if reader.at != bytes.len() {
            return Err(reader.error("unexpected data after the document"));
        }
        Ok(value)
    }

    /// Returns the member named `key` of an object.
    pub(crate) fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Returns the member named `key` if it is a string.
    pub(crate) fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Self::String(text)) => Some(text),
            _ => None,
        }
    }

    /// Returns the member named `key` if it is a number.
    pub(crate) fn number(&self, key: &str) -> Option<f64> {
        match self.get(key) {
            Some(Self::Number(number)) => Some(*number),
            _ => None,
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn error(&self, problem: &'static str) -> ParseError {
        ParseError::new(problem, self.at)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn expect(&mut self, byte: u8, problem: &'static str) -> Result<(), ParseError> {
        self.skip_space();
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error(problem))
        }
    }

    fn value(&mut self, depth: usize) -> Result<Json, ParseError> {
        if depth > MAX_DEPTH {
            return Err(self.error("nested too deeply"));
        }
        self.skip_space();
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(Json::String),
            Some(b't') => self.word(b"true", Json::Bool(true)),
            Some(b'f') => self.word(b"false", Json::Bool(false)),
            Some(b'n') => self.word(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.error("expected a value")),
            None => Err(self.error("unexpected end")),
        }
    }

    fn word(&mut self, word: &[u8], value: Json) -> Result<Json, ParseError> {
        if self.bytes.get(self.at..self.at + word.len()) == Some(word) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(self.error("expected true, false or null"))
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.at += 1;
        let mut members: Vec<(String, Json)> = Vec::new();
        self.skip_space();
        if self.peek() == Some(b'}') {
            self.at += 1;
            return Ok(Json::Object(members));
        }
        loop {
            self.skip_space();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected a member name"));
            }
            let name: String = self.string()?;
            self.expect(b':', "expected `:`")?;
            let value: Json = self.value(depth + 1)?;
            members.push((name, value));
            self.skip_space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => {
                    self.at += 1;
                    return Ok(Json::Object(members));
                }
                _ => return Err(self.error("expected `,` or `}`")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, ParseError> {
        self.at += 1;
        let mut items: Vec<Json> = Vec::new();
        self.skip_space();
        if self.peek() == Some(b']') {
            self.at += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value(depth + 1)?);
            self.skip_space();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.error("expected `,` or `]`")),
            }
        }
    }

    fn number(&mut self) -> Result<Json, ParseError> {
        let start: usize = self.at;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.at += 1;
        }
        core::str::from_utf8(&self.bytes[start..self.at])
            .ok()
            .and_then(|text| text.parse::<f64>().ok())
            .map(Json::Number)
            .ok_or_else(|| ParseError::new("malformed number", start))
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.at += 1;
        let mut text: Vec<u8> = Vec::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.error("unterminated string"));
            };
            self.at += 1;
            match byte {
                b'"' => break,
                b'\\' => self.escape(&mut text)?,
                0..0x20 => return Err(self.error("control character in string")),
                _ => text.push(byte),
            }
        }
        String::from_utf8(text).map_err(|_| self.error("string is not UTF-8"))
    }

    fn escape(&mut self, text: &mut Vec<u8>) -> Result<(), ParseError> {
        let Some(byte) = self.peek() else {
            return Err(self.error("unterminated string"));
        };
        self.at += 1;
        let simple: Option<u8> = match byte {
            b'"' => Some(b'"'),
            b'\\' => Some(b'\\'),
            b'/' => Some(b'/'),
            b'b' => Some(0x08),
            b'f' => Some(0x0c),
            b'n' => Some(b'\n'),
            b'r' => Some(b'\r'),
            b't' => Some(b'\t'),
            _ => None,
        };
        if let Some(simple) = simple {
            text.push(simple);
            return Ok(());
        }
        if byte != b'u' {
            return Err(self.error("unknown escape"));
        }
        let mut code: u32 = self.hex4()?;
        if (0xd800..0xdc00).contains(&code) {
            // A high surrogate must be followed by `\u` and a low one.
            if self.bytes.get(self.at..self.at + 2) != Some(b"\\u") {
                return Err(self.error("lone surrogate"));
            }
            self.at += 2;
            let low: u32 = self.hex4()?;
            if !(0xdc00..0xe000).contains(&low) {
                return Err(self.error("lone surrogate"));
            }
            code = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
        }
        let letter: char = char::from_u32(code).ok_or_else(|| self.error("lone surrogate"))?;
        let mut buffer: [u8; 4] = [0; 4];
        text.extend_from_slice(letter.encode_utf8(&mut buffer).as_bytes());
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, ParseError> {
        let digits: &[u8] = self
            .bytes
            .get(self.at..self.at + 4)
            .ok_or_else(|| self.error("short \\u escape"))?;
        let mut code: u32 = 0;
        for &digit in digits {
            let value: u32 = char::from(digit)
                .to_digit(16)
                .ok_or_else(|| self.error("bad \\u escape"))?;
            code = code * 16 + value;
        }
        self.at += 4;
        Ok(code)
    }
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::*;

    fn text(value: &str) -> Json {
        Json::String(String::from(value))
    }

    #[test]
    fn reads_every_kind_of_value() {
        let value = Json::parse(br#" {"a": [1, -2.5e1, true, false, null, "x"], "b": {}} "#)
            .expect("valid");
        assert_eq!(
            value.get("a"),
            Some(&Json::Array(vec![
                Json::Number(1.0),
                Json::Number(-25.0),
                Json::Bool(true),
                Json::Bool(false),
                Json::Null,
                text("x"),
            ]))
        );
        assert_eq!(value.get("b"), Some(&Json::Object(Vec::new())));
        assert_eq!(value.get("c"), None);
    }

    #[test]
    fn decodes_escapes_and_utf8() {
        let value = Json::parse(r#"["Tit\u00e4\n\"/\\", "\ud83d\udc26", "Rödhake"]"#.as_bytes())
            .expect("valid");
        assert_eq!(
            value,
            Json::Array(vec![text("Titä\n\"/\\"), text("🐦"), text("Rödhake")])
        );
    }

    #[test]
    fn rejects_broken_documents() {
        for body in [
            &b""[..],
            b"[1,]",
            b"[1 2]",
            b"{\"a\" 1}",
            b"\"open",
            b"\"\\ud800\"",
            b"\"\\q\"",
            b"[1] x",
            b"tru",
            b"\"\x01\"",
            b"\"\xff\"",
            b"--1",
        ] {
            assert!(
                Json::parse(body).is_err(),
                "{:?}",
                core::str::from_utf8(body)
            );
        }
    }

    #[test]
    fn limits_nesting() {
        let deep: Vec<u8> = core::iter::repeat_n(b'[', 100)
            .chain(core::iter::repeat_n(b']', 100))
            .collect();
        assert!(Json::parse(&deep).is_err());
        let fine: Vec<u8> = core::iter::repeat_n(b'[', 10)
            .chain(core::iter::repeat_n(b']', 10))
            .collect();
        assert!(Json::parse(&fine).is_ok());
    }
}
