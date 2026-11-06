//! The network to join.

/// A WPA2 network's name and password.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WifiCredentials<'a> {
    /// The network's name (SSID), at most 32 bytes.
    pub ssid: &'a str,
    /// The network's password, 8 to 63 bytes, or empty for an open network.
    pub password: &'a str,
}

/// Shows the network's name only, so a log never carries the password.
impl core::fmt::Debug for WifiCredentials<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("WifiCredentials")
            .field("ssid", &self.ssid)
            .finish_non_exhaustive()
    }
}
