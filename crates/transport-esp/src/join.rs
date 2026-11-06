//! Joining the Wi-Fi, and what to remember to join faster next time.

use embassy_net::{Config, DhcpConfig, Ipv4Cidr, Stack, StaticConfigV4};
use embassy_time::Duration;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{self, AuthenticationMethodConfig, WifiController};
use panel_wake::Rejoin;

use crate::transport_error::TransportError;
use crate::wifi_credentials::WifiCredentials;

/// The access point the panel joined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Joined {
    /// The access point's MAC address.
    pub bssid: [u8; 6],
    /// The access point's channel.
    pub channel: u8,
}

/// Joins the network in `credentials` within `timeout`.
///
/// With `rejoin`, the radio goes straight to the remembered access point on its channel instead
/// of scanning them all. If that access point has moved, the join fails; join again without
/// `rejoin` to scan.
///
/// # Errors
///
/// Returns [`TransportError::Credentials`] when the name or password cannot be used,
/// [`TransportError::Radio`] when the radio refuses its configuration, or
/// [`TransportError::Join`] when the access point turns the panel away or does not answer.
pub async fn join(
    controller: &mut WifiController<'_>,
    credentials: WifiCredentials<'_>,
    rejoin: Option<&Rejoin>,
    timeout: Duration,
) -> Result<Joined, TransportError> {
    let authentication: AuthenticationMethodConfig = if credentials.password.is_empty() {
        AuthenticationMethodConfig::Open
    } else {
        AuthenticationMethodConfig::Wpa2Personal(
            credentials
                .password
                .try_into()
                .map_err(|_| TransportError::Credentials)?,
        )
    };
    let mut station: StationConfig = StationConfig::default()
        .with_ssid(
            credentials
                .ssid
                .try_into()
                .map_err(|_| TransportError::Credentials)?,
        )
        .with_authentication(authentication);
    if let Some(rejoin) = rejoin {
        station = station
            .with_bssid(rejoin.bssid)
            .with_channel(rejoin.channel);
    }
    controller
        .set_config(&wifi::Config::Station(station))
        .map_err(|_| TransportError::Radio)?;
    let connected = embassy_time::with_timeout(timeout, controller.connect_async())
        .await
        .map_err(|_| TransportError::Join)?
        .map_err(|_| TransportError::Join)?;
    Ok(Joined {
        bssid: connected.bssid,
        channel: connected.channel,
    })
}

/// The network stack's configuration: the remembered lease as a static address, which needs no
/// DHCP round trip, or DHCP when there is nothing remembered.
#[must_use]
pub fn network_config(rejoin: Option<&Rejoin>) -> Config {
    match rejoin {
        Some(rejoin) => Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(rejoin.address, rejoin.prefix_len),
            gateway: rejoin.gateway,
            dns_servers: Default::default(),
        }),
        None => Config::dhcpv4(DhcpConfig::default()),
    }
}

/// What to remember for the next wake: the access point joined and the stack's address.
/// Returns `None` while the stack has no address.
#[must_use]
pub fn rejoin(joined: Joined, stack: Stack<'_>) -> Option<Rejoin> {
    let config: StaticConfigV4 = stack.config_v4()?;
    Some(Rejoin {
        bssid: joined.bssid,
        channel: joined.channel,
        address: config.address.address(),
        prefix_len: config.address.prefix_len(),
        gateway: config.gateway,
    })
}
