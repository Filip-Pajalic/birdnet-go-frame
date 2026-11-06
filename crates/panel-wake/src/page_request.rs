//! The requests a panel sends.

use alloc::format;
use alloc::string::String;
use core::fmt::Write;

use frame_core::{PanelSize, Rotation};

use crate::page_view::PageView;

/// The path of the firmware the server offers to EE02 frames.
pub const FIRMWARE_PATH: &str = "/firmware/ee02-thin.bin";

/// Everything a panel says when it asks for its page.
///
/// ```
/// use frame_core::PanelSize;
/// use panel_wake::{PageRequest, PageView};
///
/// let request = PageRequest {
///     view: PageView::Week,
///     frame: Some("hall"),
///     battery_millivolts: Some(3912),
///     firmware: Some("0.2.0"),
///     ..PageRequest::new("192.168.1.20:8081", PanelSize::Inches13Point3)
/// };
/// let http: String = request.to_http();
/// assert!(http.starts_with("GET /panel?panel=13.3&view=week&frame=hall HTTP/1.1\r\n"));
/// assert!(http.contains("X-Frame-Battery-mV: 3912\r\n"));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageRequest<'a> {
    /// The server's address as `Host`, e.g. `192.168.1.20:8081`.
    pub host: &'a str,
    /// The panel, so the server never sends a page laid out for another.
    pub panel: PanelSize,
    /// The rotation, or `None` to leave it to the server's `ROTATION`.
    pub rotation: Option<Rotation>,
    /// The ETag of the page on the glass, so an unchanged page costs only a `304`.
    pub etag: Option<&'a str>,
    /// The page to show; [`PageView::Today`] is left out of the query.
    pub view: PageView,
    /// The frame's name, so the server can give each frame its own layout.
    pub frame: Option<&'a str>,
    /// The battery's voltage, for the server's sensors.
    pub battery_millivolts: Option<u16>,
    /// The firmware's version, so the server can offer a newer one.
    pub firmware: Option<&'a str>,
}

impl<'a> PageRequest<'a> {
    /// A request for `panel`'s default page from `host`, with nothing else said.
    #[must_use]
    pub fn new(host: &'a str, panel: PanelSize) -> Self {
        Self {
            host,
            panel,
            rotation: None,
            etag: None,
            view: PageView::Today,
            frame: None,
            battery_millivolts: None,
            firmware: None,
        }
    }

    /// The request as HTTP/1.1, asking for the page compressed and the connection closed.
    #[must_use]
    pub fn to_http(&self) -> String {
        let mut query: String = format!("panel={}", self.panel);
        if let Some(rotation) = self.rotation {
            let _ = write!(query, "&rotation={}", rotation.degrees());
        }
        if self.view != PageView::Today {
            let _ = write!(query, "&view={}", self.view);
        }
        if let Some(frame) = self.frame.filter(|frame| !frame.is_empty()) {
            query.push_str("&frame=");
            push_encoded(&mut query, frame);
        }
        let mut headers: String = String::new();
        if let Some(etag) = self.etag {
            let _ = write!(headers, "If-None-Match: {etag}\r\n");
        }
        if let Some(millivolts) = self.battery_millivolts {
            let _ = write!(headers, "X-Frame-Battery-mV: {millivolts}\r\n");
        }
        if let Some(firmware) = self.firmware {
            let _ = write!(headers, "X-Frame-Firmware: {firmware}\r\n");
        }
        format!(
            "GET /panel?{query} HTTP/1.1\r\nHost: {host}\r\nAccept-Encoding: deflate\r\n\
             {headers}Connection: close\r\n\r\n",
            host = self.host
        )
    }
}

/// Returns the HTTP request for `panel`'s page from `host`, e.g. `192.168.1.20:8081`.
///
/// It names the panel, so the server never sends a page laid out for another; names the
/// rotation if given, or leaves it to the server's `ROTATION`; asks for the page compressed; and
/// sends `etag`, so an unchanged page costs only a `304`. [`PageRequest`] says more.
#[must_use]
pub fn page_request(
    host: &str,
    panel: PanelSize,
    rotation: Option<Rotation>,
    etag: Option<&str>,
) -> String {
    PageRequest {
        rotation,
        etag,
        ..PageRequest::new(host, panel)
    }
    .to_http()
}

/// Returns the HTTP request for the firmware the server offers, from `host`.
#[must_use]
pub fn firmware_request(host: &str) -> String {
    format!("GET {FIRMWARE_PATH} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
}

/// Appends `text` to a query, percent-encoding all but letters, digits, `-`, `_` and `.`.
fn push_encoded(query: &mut String, text: &str) {
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            query.push(char::from(byte));
        } else {
            let _ = write!(query, "%{byte:02X}");
        }
    }
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

    #[test]
    fn says_the_view_the_frame_the_battery_and_the_firmware() {
        let request = PageRequest {
            view: PageView::Rarest,
            frame: Some("desk 2"),
            battery_millivolts: Some(3_700),
            firmware: Some("0.2.0"),
            ..PageRequest::new("10.0.0.2:8080", PanelSize::Inches13Point3)
        };
        assert_eq!(
            request.to_http(),
            "GET /panel?panel=13.3&view=rarest&frame=desk%202 HTTP/1.1\r\nHost: 10.0.0.2:8080\r\n\
             Accept-Encoding: deflate\r\nX-Frame-Battery-mV: 3700\r\nX-Frame-Firmware: 0.2.0\r\n\
             Connection: close\r\n\r\n"
        );
        let today = PageRequest {
            frame: Some(""),
            ..request
        };
        let http: String = PageRequest {
            view: PageView::Today,
            ..today
        }
        .to_http();
        assert!(
            http.starts_with("GET /panel?panel=13.3 HTTP/1.1\r\n"),
            "{http}"
        );
    }

    #[test]
    fn asks_for_the_firmware() {
        assert_eq!(
            firmware_request("10.0.0.2:8080"),
            "GET /firmware/ee02-thin.bin HTTP/1.1\r\nHost: 10.0.0.2:8080\r\nConnection: close\r\n\r\n"
        );
    }
}
