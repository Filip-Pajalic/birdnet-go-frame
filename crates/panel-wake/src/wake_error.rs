//! Why a wake did not end with a new page.

use core::fmt;

use frame_core::InflateError;

/// Why a wake ended without a page to show.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum WakeError {
    /// The response's head is not HTTP the panel can read.
    BadHead,
    /// The server answered with this status: 503 before its first page, 400 for a bad request.
    Status(u16),
    /// The body did not unpack.
    Inflate(InflateError),
    /// The page is not as long as the panel needs: it is laid out for another panel.
    WrongLength {
        /// Bytes the panel needs.
        expected: usize,
        /// Bytes that came.
        got: usize,
    },
    /// The page's CRC-32 does not match its ETag: damaged on the way.
    Damaged,
    /// The connection failed before a response, e.g. no Wi-Fi.
    Network,
    /// A new page came but the panel could not show it.
    Panel,
}

impl fmt::Display for WakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadHead => formatter.write_str("the frame's answer is not HTTP"),
            Self::Status(503) => formatter.write_str("the frame has not drawn a page yet"),
            Self::Status(status) => write!(formatter, "the frame answered {status}"),
            Self::Inflate(error) => write!(formatter, "the page did not unpack: {error}"),
            Self::WrongLength { expected, got } => write!(
                formatter,
                "the page is {got} bytes, this panel takes {expected}: is the frame set up for \
                 another panel?"
            ),
            Self::Damaged => formatter.write_str("the page arrived damaged"),
            Self::Network => formatter.write_str("cannot reach the frame"),
            Self::Panel => formatter.write_str("the panel did not take the page"),
        }
    }
}

impl core::error::Error for WakeError {}
