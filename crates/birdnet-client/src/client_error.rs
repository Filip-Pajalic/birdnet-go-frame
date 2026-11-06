//! Why a request to BirdNET-Go failed.

use core::fmt;

use birdnet_model::ParseError;

/// Why a request to BirdNET-Go failed.
#[derive(Debug)]
#[must_use]
#[non_exhaustive]
pub enum ClientError<E> {
    /// No response: DNS, connection or timeout.
    Transport(E),
    /// BirdNET-Go answered 401 or 403: its private mode needs a login, and the client has no
    /// password ([`crate::BirdnetClient::with_login`]).
    Unauthorized,
    /// The client tried to log in and BirdNET-Go refused: a wrong password, or password login
    /// is off.
    LoginRefused,
    /// BirdNET-Go answered with another non-success status.
    Status(u16),
    /// The body was not the documented JSON.
    Parse(ParseError),
}

impl<E: fmt::Display> fmt::Display for ClientError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "cannot reach BirdNET-Go: {error}"),
            Self::Unauthorized => formatter
                .write_str("BirdNET-Go requires a login (private mode): set BIRDNET_PASSWORD"),
            Self::LoginRefused => {
                formatter.write_str("BirdNET-Go refused the login: check BIRDNET_PASSWORD")
            }
            Self::Status(status) => write!(formatter, "BirdNET-Go answered HTTP {status}"),
            Self::Parse(error) => write!(formatter, "{error}"),
        }
    }
}

impl<E: core::error::Error> core::error::Error for ClientError<E> {}
