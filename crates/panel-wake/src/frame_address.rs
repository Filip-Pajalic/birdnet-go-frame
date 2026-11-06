//! Where the panel finds the frame server.

use core::fmt;
use core::net::{Ipv4Addr, SocketAddrV4};
use core::str::FromStr;

/// The frame server's IPv4 address and port, e.g. `192.168.1.20:8081`.
///
/// The panel takes an address rather than a name: it skips a DNS lookup on every wake, and a
/// server on a home network usually has a fixed address anyway.
///
/// ```
/// use panel_wake::FrameAddress;
///
/// let address: FrameAddress = "192.168.1.20:8081".parse().expect("an address");
/// assert_eq!(address.to_string(), "192.168.1.20:8081");
/// let default_port: FrameAddress = "10.0.0.5".parse().expect("an address");
/// assert_eq!(default_port.port(), 8080);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameAddress(SocketAddrV4);

impl FrameAddress {
    /// The port the server listens on unless told otherwise.
    pub const DEFAULT_PORT: u16 = 8080;

    /// Returns the address for `ip` and `port`.
    #[must_use]
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        Self(SocketAddrV4::new(ip, port))
    }

    /// The server's IPv4 address.
    #[must_use]
    pub fn ip(&self) -> Ipv4Addr {
        *self.0.ip()
    }

    /// The server's TCP port.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.0.port()
    }
}

/// Writes `address:port`, which is also the `Host` header the panel sends.
impl fmt::Display for FrameAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// The error returned when a frame address is not `a.b.c.d` or `a.b.c.d:port`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub struct ParseFrameAddressError;

impl fmt::Display for ParseFrameAddressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the frame's address must be an IPv4 address with an optional port")
    }
}

impl core::error::Error for ParseFrameAddressError {}

impl FromStr for FrameAddress {
    type Err = ParseFrameAddressError;

    /// Reads `a.b.c.d:port`, or `a.b.c.d` for port 8080. An `http://` prefix and a trailing `/`
    /// are allowed, since that is how the address is usually copied from a browser.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text: &str = text.trim();
        let text: &str = text.strip_prefix("http://").unwrap_or(text);
        let text: &str = text.strip_suffix('/').unwrap_or(text);
        let (ip, port) = match text.split_once(':') {
            Some((ip, port)) => (ip, port.parse().map_err(|_| ParseFrameAddressError)?),
            None => (text, Self::DEFAULT_PORT),
        };
        let ip: Ipv4Addr = ip.parse().map_err(|_| ParseFrameAddressError)?;
        Ok(Self::new(ip, port))
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn reads_what_people_paste() {
        let expected = FrameAddress::new(Ipv4Addr::new(192, 168, 1, 20), 8081);
        for text in [
            "192.168.1.20:8081",
            " http://192.168.1.20:8081/ ",
            "http://192.168.1.20:8081",
        ] {
            assert_eq!(text.parse(), Ok(expected), "{text}");
        }
        assert_eq!(expected.to_string(), "192.168.1.20:8081");
    }

    #[test]
    fn refuses_names_and_bad_ports() {
        for text in ["frame.local:8080", "192.168.1.20:http", "192.168.1:80", ""] {
            assert_eq!(
                text.parse::<FrameAddress>(),
                Err(ParseFrameAddressError),
                "{text}"
            );
        }
    }
}
