//! Why the weather could not be read.

use core::fmt;

/// Why the weather could not be read.
#[derive(Debug)]
#[must_use]
pub enum WeatherError<E> {
    /// No response: DNS, connection or timeout.
    Transport(E),
    /// Open-Meteo answered with a status other than success.
    Status(u16),
    /// The body was not a forecast with the fields asked for.
    Unreadable,
}

impl<E: fmt::Display> fmt::Display for WeatherError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(formatter, "cannot reach Open-Meteo: {error}"),
            Self::Status(status) => write!(formatter, "Open-Meteo answered HTTP {status}"),
            Self::Unreadable => formatter.write_str("Open-Meteo sent an unreadable forecast"),
        }
    }
}

impl<E: core::error::Error> core::error::Error for WeatherError<E> {}
