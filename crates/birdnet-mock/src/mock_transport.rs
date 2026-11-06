//! Plugs the simulated station into a client without any networking.

use birdnet_client::{HttpResponse, HttpTransport};

use crate::mock_birdnet::MockBirdnet;

/// An [`HttpTransport`] answered by a [`MockBirdnet`] in-process.
///
/// The URL's host is ignored; only the path and query matter. `clock` returns the current unix
/// time in seconds, so tests can pin it.
#[derive(Clone, Debug)]
pub struct MockTransport<C> {
    station: MockBirdnet,
    clock: C,
}

impl<C: FnMut() -> i64> MockTransport<C> {
    /// Creates a transport for a station at UTC, reading the time from `clock`.
    #[must_use]
    pub fn new(clock: C) -> Self {
        Self {
            station: MockBirdnet::new(0),
            clock,
        }
    }

    /// Creates a transport for a station `utc_offset_minutes` east of UTC.
    #[must_use]
    pub fn with_utc_offset(clock: C, utc_offset_minutes: i32) -> Self {
        Self {
            station: MockBirdnet::new(utc_offset_minutes),
            clock,
        }
    }
}

impl<C: FnMut() -> i64> HttpTransport for MockTransport<C> {
    type Error = core::convert::Infallible;

    fn get(&mut self, url: &str) -> Result<HttpResponse, Self::Error> {
        let after_scheme: &str = url.split_once("://").map_or(url, |(_, rest)| rest);
        let path: &str = after_scheme
            .find('/')
            .map_or("/", |start| &after_scheme[start..]);
        Ok(self.station.respond(path, (self.clock)()))
    }
}
