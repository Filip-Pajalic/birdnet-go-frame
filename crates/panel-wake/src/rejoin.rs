//! What the panel remembers about its Wi-Fi to join faster next time.

use core::net::Ipv4Addr;

/// The access point and address lease of the last good wake.
///
/// A full join scans every channel and then waits for DHCP, which together take a few seconds
/// of radio time. With this, the next wake joins the same access point on its known channel and
/// reuses the address straight away, which takes well under a second. The firmware forgets it
/// whenever a wake that used it fails, so a moved access point or an expired lease costs one
/// slow wake, not a stuck panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rejoin {
    /// The access point's MAC address.
    pub bssid: [u8; 6],
    /// The access point's channel, 1 to 14.
    pub channel: u8,
    /// The panel's address from the last DHCP lease.
    pub address: Ipv4Addr,
    /// The lease's network prefix length, e.g. 24.
    pub prefix_len: u8,
    /// The lease's gateway, if it gave one.
    pub gateway: Option<Ipv4Addr>,
}

impl Rejoin {
    /// Bytes [`to_bytes`](Self::to_bytes) gives.
    pub const BYTES: usize = 17;

    /// The record as bytes, e.g. for RTC memory.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut bytes: [u8; Self::BYTES] = [0; Self::BYTES];
        bytes[..6].copy_from_slice(&self.bssid);
        bytes[6] = self.channel;
        bytes[7..11].copy_from_slice(&self.address.octets());
        bytes[11] = self.prefix_len;
        bytes[12] = u8::from(self.gateway.is_some());
        if let Some(gateway) = self.gateway {
            bytes[13..17].copy_from_slice(&gateway.octets());
        }
        bytes
    }

    /// Reads [`to_bytes`](Self::to_bytes)' bytes. Returns `None` when they cannot be a record:
    /// all zeros, as on a first boot, or a channel or prefix out of range.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; Self::BYTES]) -> Option<Self> {
        let channel: u8 = bytes[6];
        let prefix_len: u8 = bytes[11];
        if !(1..=14).contains(&channel) || !(1..=32).contains(&prefix_len) {
            return None;
        }
        let mut bssid: [u8; 6] = [0; 6];
        bssid.copy_from_slice(&bytes[..6]);
        let gateway: Option<Ipv4Addr> =
            (bytes[12] == 1).then(|| Ipv4Addr::new(bytes[13], bytes[14], bytes[15], bytes[16]));
        Some(Self {
            bssid,
            channel,
            address: Ipv4Addr::new(bytes[7], bytes[8], bytes[9], bytes[10]),
            prefix_len,
            gateway,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn round_trips_and_refuses_empty_memory() {
        let rejoin = Rejoin {
            bssid: [0x3c, 0x84, 0x6a, 0x01, 0x02, 0x03],
            channel: 6,
            address: Ipv4Addr::new(192, 168, 1, 77),
            prefix_len: 24,
            gateway: Some(Ipv4Addr::new(192, 168, 1, 1)),
        };
        assert_eq!(Rejoin::from_bytes(&rejoin.to_bytes()), Some(rejoin));
        let no_gateway = Rejoin {
            gateway: None,
            ..rejoin
        };
        assert_eq!(Rejoin::from_bytes(&no_gateway.to_bytes()), Some(no_gateway));
        assert_eq!(Rejoin::from_bytes(&[0; Rejoin::BYTES]), None);
    }
}
