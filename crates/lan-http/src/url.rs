//! Splitting an `http://` URL.

use crate::http_error::HttpError;

/// The parts of an `http://` URL a request needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Url<'a> {
    /// `host:port` as written, for the `Host` header.
    pub(crate) authority: &'a str,
    /// Host name or address, without brackets around IPv6.
    pub(crate) host: &'a str,
    pub(crate) port: u16,
    /// Path and query, at least `/`.
    pub(crate) target: &'a str,
}

impl<'a> Url<'a> {
    pub(crate) fn parse(text: &'a str) -> Result<Self, HttpError> {
        let text: &str = text.trim();
        let rest: &str = match text.get(..7) {
            Some(scheme) if scheme.eq_ignore_ascii_case("http://") => &text[7..],
            _ if text
                .get(..8)
                .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://")) =>
            {
                return Err(HttpError::BadUrl(
                    "https is not supported; use the plain http:// address on your network",
                ));
            }
            _ => return Err(HttpError::BadUrl("expected an http:// URL")),
        };
        let split: usize = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let (authority, tail) = rest.split_at(split);
        let target: &str = match tail.split('#').next() {
            Some("") | None => "/",
            Some(target) if target.starts_with('?') => {
                return Err(HttpError::BadUrl("expected a path before the query"));
            }
            Some(target) => target,
        };
        if authority.is_empty() {
            return Err(HttpError::BadUrl("missing host"));
        }
        if authority.contains('@') {
            return Err(HttpError::BadUrl("user names in URLs are not supported"));
        }
        let (host, port_text): (&str, Option<&str>) =
            if let Some(bracketed) = authority.strip_prefix('[') {
                let (host, after) = bracketed
                    .split_once(']')
                    .ok_or(HttpError::BadUrl("unclosed `[` in host"))?;
                (host, after.strip_prefix(':'))
            } else {
                match authority.rsplit_once(':') {
                    Some((host, port)) => (host, Some(port)),
                    None => (authority, None),
                }
            };
        let port: u16 = match port_text {
            None => 80,
            Some(port) => port.parse().map_err(|_| HttpError::BadUrl("bad port"))?,
        };
        Ok(Self {
            authority,
            host,
            port,
            target,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn splits_urls() {
        let url = Url::parse("http://birdnet:8080/api/v2/x?a=1#frag").expect("valid");
        assert_eq!(
            url,
            Url {
                authority: "birdnet:8080",
                host: "birdnet",
                port: 8080,
                target: "/api/v2/x?a=1"
            }
        );
        let url = Url::parse("HTTP://[::1]").expect("valid");
        assert_eq!((url.host, url.port, url.target), ("::1", 80, "/"));
    }

    #[test]
    fn refuses_what_it_cannot_fetch() {
        for text in [
            "https://x/",
            "ftp://x/",
            "http://",
            "http://u@x/",
            "http://x:http/",
            "http://x?q",
        ] {
            assert!(Url::parse(text).is_err(), "{text}");
        }
    }
}
