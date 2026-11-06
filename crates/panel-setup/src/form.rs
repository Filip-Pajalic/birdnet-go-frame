//! Reading an HTML form's `application/x-www-form-urlencoded` body.

use alloc::string::String;
use alloc::vec::Vec;

/// A form value that is not valid percent-encoded UTF-8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadEncoding;

/// The form's fields in order, as names and decoded values. Names are taken as they are; the
/// setup page's names need no decoding.
pub fn fields(body: &[u8]) -> impl Iterator<Item = (&str, Result<String, BadEncoding>)> {
    body.split(|&byte| byte == b'&')
        .filter(|field| !field.is_empty())
        .map(|field| {
            let (name, value): (&[u8], &[u8]) = match field.iter().position(|&byte| byte == b'=') {
                Some(at) => (&field[..at], &field[at + 1..]),
                None => (field, &[]),
            };
            (core::str::from_utf8(name).unwrap_or(""), decode(value))
        })
}

/// Decodes `+` as a space and `%XX` as its byte, then checks the result is UTF-8.
fn decode(value: &[u8]) -> Result<String, BadEncoding> {
    let mut bytes: Vec<u8> = Vec::with_capacity(value.len());
    let mut rest: &[u8] = value;
    while let Some((&byte, after)) = rest.split_first() {
        rest = after;
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let hex: &[u8] = rest
                    .get(..2)
                    .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit))
                    .ok_or(BadEncoding)?;
                let text: &str = core::str::from_utf8(hex).map_err(|_| BadEncoding)?;
                bytes.push(u8::from_str_radix(text, 16).map_err(|_| BadEncoding)?);
                rest = &rest[2..];
            }
            _ => bytes.push(byte),
        }
    }
    String::from_utf8(bytes).map_err(|_| BadEncoding)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decodes_what_browsers_send() {
        let read: Vec<(&str, Result<String, BadEncoding>)> =
            fields(b"ssid=Caf%C3%A9+Wi-Fi&password=a%26b%3Dc&&open&bad=%4").collect();
        assert_eq!(read[0], ("ssid", Ok(String::from("Café Wi-Fi"))));
        assert_eq!(read[1], ("password", Ok(String::from("a&b=c"))));
        assert_eq!(read[2], ("open", Ok(String::new())));
        assert_eq!(read[3], ("bad", Err(BadEncoding)));
        assert_eq!(decode(b"%ff"), Err(BadEncoding));
        assert_eq!(decode(b"%+1"), Err(BadEncoding));
    }
}
