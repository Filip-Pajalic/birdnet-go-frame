//! The request a panel sends.

use alloc::format;
use alloc::string::String;

use frame_core::{PanelSize, Rotation};

/// Returns the HTTP request for `panel`'s page from `host`, e.g. `192.168.1.20:8081`.
///
/// It names the panel, so the server never sends a page laid out for another; names the
/// rotation if given, or leaves it to the server's `ROTATION`; asks for the page compressed; and
/// sends `etag`, so an unchanged page costs only a `304`.
#[must_use]
pub fn page_request(
    host: &str,
    panel: PanelSize,
    rotation: Option<Rotation>,
    etag: Option<&str>,
) -> String {
    let turn: String = rotation.map_or_else(String::new, |rotation| {
        format!("&rotation={}", rotation.degrees())
    });
    let validator: String =
        etag.map_or_else(String::new, |etag| format!("If-None-Match: {etag}\r\n"));
    format!(
        "GET /panel?panel={panel}{turn} HTTP/1.1\r\nHost: {host}\r\n\
         Accept-Encoding: deflate\r\n{validator}Connection: close\r\n\r\n"
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn names_the_panel_and_the_last_page() {
        assert_eq!(
            page_request(
                "frame:8081",
                PanelSize::Inches13Point3,
                Some(Rotation::Degrees90),
                Some("\"b35332bb-z\"")
            ),
            "GET /panel?panel=13.3&rotation=90 HTTP/1.1\r\nHost: frame:8081\r\n\
             Accept-Encoding: deflate\r\nIf-None-Match: \"b35332bb-z\"\r\nConnection: close\r\n\r\n"
        );
        let first: String = page_request("frame", PanelSize::Inches7Point3, None, None);
        assert!(first.starts_with("GET /panel?panel=7.3 HTTP/1.1\r\n"));
        assert!(!first.contains("If-None-Match"));
    }
}
