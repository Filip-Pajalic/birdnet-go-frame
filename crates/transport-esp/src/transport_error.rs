//! What can go wrong between the radio and the response.

use core::fmt;

/// Why the panel could not get a response from the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum TransportError {
    /// The network's name or password does not fit the radio's limits.
    Credentials,
    /// The radio refused its configuration.
    Radio,
    /// The access point did not let the panel join, or did not answer in time.
    Join,
    /// No address came from DHCP in time.
    Address,
    /// The frame did not accept the connection.
    Connect,
    /// The connection broke or stalled before the response was complete.
    Io,
    /// The response is longer than the panel is willing to hold.
    TooLong,
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Credentials => "the Wi-Fi name or password is too long or too short",
            Self::Radio => "the radio refused its configuration",
            Self::Join => "could not join the Wi-Fi",
            Self::Address => "no address from DHCP",
            Self::Connect => "the frame refused the connection",
            Self::Io => "the connection to the frame broke",
            Self::TooLong => "the frame's answer is too long",
        })
    }
}

impl core::error::Error for TransportError {}
