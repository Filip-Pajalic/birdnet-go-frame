//! What can go wrong driving the panel.

use core::fmt;

use embedded_hal::{digital, spi};

/// Why the panel could not be driven.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
#[non_exhaustive]
pub enum PanelError {
    /// The SPI bus failed.
    Bus(spi::ErrorKind),
    /// A pin could not be set or read.
    Pin(digital::ErrorKind),
    /// The controllers stayed busy after a command.
    StayedBusy,
    /// The bytes are not one page: 960,000 of them.
    WrongLength(usize),
    /// BUSY never went low after the refresh command: the refresh did not start.
    RefreshNotStarted,
    /// BUSY stayed low for over a minute.
    RefreshTimedOut,
    /// BUSY came back after fewer milliseconds than a real refresh takes.
    RefreshTooShort(u32),
}

impl fmt::Display for PanelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bus(kind) => write!(formatter, "SPI bus failed: {kind:?}"),
            Self::Pin(kind) => write!(formatter, "pin failed: {kind:?}"),
            Self::StayedBusy => formatter.write_str("panel stayed busy"),
            Self::WrongLength(length) => {
                write!(formatter, "a page is 960000 bytes, not {length}")
            }
            Self::RefreshNotStarted => formatter.write_str("panel did not start refreshing"),
            Self::RefreshTimedOut => formatter.write_str("panel refresh took over a minute"),
            Self::RefreshTooShort(milliseconds) => {
                write!(
                    formatter,
                    "panel released BUSY after only {milliseconds} ms"
                )
            }
        }
    }
}

impl core::error::Error for PanelError {}
