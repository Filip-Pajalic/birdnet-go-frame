//! The setup hotspot: the frame's own Wi-Fi network, and a look at the networks around it.

use alloc::string::String;
use alloc::vec::Vec;

use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StaticConfigV4};
use esp_radio::wifi::ap::AccessPointConfig;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{self, AuthenticationMethodConfig, WifiController};

use crate::transport_error::TransportError;

/// The hotspot's own address, which phones get as their gateway and DNS server.
pub const HOTSPOT_ADDRESS: Ipv4Address = Ipv4Address::new(192, 168, 4, 1);

/// The most networks [`scan`] returns.
const SCAN_MAX: usize = 20;

/// Returns the names of the Wi-Fi networks in range, strongest first, without duplicates or
/// hidden ones. Call it before [`open_hotspot`]: it puts the radio in station mode.
///
/// # Errors
///
/// Returns [`TransportError::Radio`] when the radio refuses to scan.
pub async fn scan(controller: &mut WifiController<'_>) -> Result<Vec<String>, TransportError> {
    controller
        .set_config(&wifi::Config::Station(StationConfig::default()))
        .map_err(|_| TransportError::Radio)?;
    let mut found = controller
        .scan_async(&ScanConfig::default().with_max(SCAN_MAX))
        .await
        .map_err(|_| TransportError::Radio)?;
    found.sort_unstable_by_key(|network| core::cmp::Reverse(network.signal_strength));
    let mut names: Vec<String> = Vec::new();
    for network in found {
        let name: &str = network.ssid.as_str();
        if !name.is_empty() && !names.iter().any(|known| known == name) {
            names.push(String::from(name));
        }
    }
    Ok(names)
}

/// Opens an open Wi-Fi network called `name`, for phones to join.
///
/// # Errors
///
/// Returns [`TransportError::Credentials`] when the name is too long, or
/// [`TransportError::Radio`] when the radio refuses its configuration.
pub fn open_hotspot(controller: &mut WifiController<'_>, name: &str) -> Result<(), TransportError> {
    let config: AccessPointConfig = AccessPointConfig::default()
        .with_ssid(name.try_into().map_err(|_| TransportError::Credentials)?)
        .with_authentication(AuthenticationMethodConfig::Open)
        .with_max_connections(4);
    controller
        .set_config(&wifi::Config::AccessPoint(config))
        .map_err(|_| TransportError::Radio)
}

/// The network stack's configuration on the hotspot: [`HOTSPOT_ADDRESS`] on a /24.
#[must_use]
pub fn hotspot_config() -> Config {
    Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(HOTSPOT_ADDRESS, 24),
        gateway: None,
        dns_servers: Default::default(),
    })
}
