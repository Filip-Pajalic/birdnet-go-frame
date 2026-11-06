//! Just enough DHCP to give a phone an address on the setup hotspot.

use alloc::vec;
use alloc::vec::Vec;
use core::net::Ipv4Addr;

/// Bytes of a DHCP message before its options, magic cookie included.
const FIXED_BYTES: usize = 240;
/// The cookie that marks DHCP options (RFC 2131).
const MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];
/// The shortest reply some clients accept.
const MIN_REPLY: usize = 300;
/// How long a lease lasts: longer than the hotspot stays open.
const LEASE_SECONDS: u32 = 3_600;
/// Addresses handed out: .2 to .9 of the hotspot's network.
const POOL: usize = 8;

/// Message types (option 53).
const DISCOVER: u8 = 1;
const OFFER: u8 = 2;
const REQUEST: u8 = 3;
const ACK: u8 = 5;

/// Option codes.
const OPTION_SUBNET: u8 = 1;
const OPTION_ROUTER: u8 = 3;
const OPTION_DNS: u8 = 6;
const OPTION_LEASE: u8 = 51;
const OPTION_TYPE: u8 = 53;
const OPTION_SERVER: u8 = 54;
const OPTION_END: u8 = 255;
const OPTION_PAD: u8 = 0;

/// A DHCP server for a small network where it is also the gateway and the DNS server, so that
/// every name a phone looks up leads to the setup page.
///
/// It answers DISCOVER with OFFER and REQUEST with ACK, gives each device the same address every
/// time, and hands out eight addresses at most, reusing the oldest when they run out. Replies
/// go to the broadcast address, port 68, since the device has no address yet.
///
/// ```
/// use core::net::Ipv4Addr;
/// use panel_setup::DhcpServer;
///
/// let mut server = DhcpServer::new(Ipv4Addr::new(192, 168, 4, 1));
/// let mut discover = vec![0_u8; 240];
/// discover[0] = 1;
/// discover[28..34].copy_from_slice(&[2, 0, 0, 0, 0, 1]);
/// discover[236..240].copy_from_slice(&[99, 130, 83, 99]);
/// discover.extend_from_slice(&[53, 1, 1, 255]);
/// let offer: Vec<u8> = server.answer(&discover).expect("an offer");
/// assert_eq!(&offer[16..20], &[192, 168, 4, 2]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DhcpServer {
    /// The server's own address, at `.1` of a /24.
    address: Ipv4Addr,
    /// Which device has each address, by MAC, in the order they were given.
    leases: [Option<[u8; 6]>; POOL],
    /// The lease to reuse next when the pool is full.
    oldest: usize,
}

impl DhcpServer {
    /// A server at `address` that hands out the next eight addresses of its /24.
    #[must_use]
    pub fn new(address: Ipv4Addr) -> Self {
        Self {
            address,
            leases: [None; POOL],
            oldest: 0,
        }
    }

    /// The reply to a client's `message`, or `None` for anything it does not answer: replies
    /// from other servers, a REQUEST meant for another server, RELEASE, garbage.
    pub fn answer(&mut self, message: &[u8]) -> Option<Vec<u8>> {
        if message.len() < FIXED_BYTES || message[0] != 1 || message[236..240] != MAGIC_COOKIE {
            return None;
        }
        let mut kind: Option<u8> = None;
        let mut server: Option<[u8; 4]> = None;
        let mut options: &[u8] = &message[FIXED_BYTES..];
        while let Some((&code, rest)) = options.split_first() {
            if code == OPTION_END {
                break;
            }
            if code == OPTION_PAD {
                options = rest;
                continue;
            }
            let (&length, rest) = rest.split_first()?;
            let value: &[u8] = rest.get(..usize::from(length))?;
            match (code, value) {
                (OPTION_TYPE, [message_type]) => kind = Some(*message_type),
                (OPTION_SERVER, [a, b, c, d]) => server = Some([*a, *b, *c, *d]),
                _ => {}
            }
            options = &rest[usize::from(length)..];
        }
        let reply_type: u8 = match kind? {
            DISCOVER => OFFER,
            REQUEST if server.is_none_or(|server| server == self.address.octets()) => ACK,
            _ => return None,
        };
        let mut mac: [u8; 6] = [0; 6];
        mac.copy_from_slice(&message[28..34]);
        let offered: Ipv4Addr = self.lease(mac);
        Some(self.reply(message, reply_type, offered))
    }

    /// The address for `mac`: the one it had, or a free one, or the oldest one.
    fn lease(&mut self, mac: [u8; 6]) -> Ipv4Addr {
        let slot: usize = match self.leases.iter().position(|lease| *lease == Some(mac)) {
            Some(slot) => slot,
            None => {
                let slot: usize = self
                    .leases
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(self.oldest);
                self.oldest = (slot + 1) % POOL;
                self.leases[slot] = Some(mac);
                slot
            }
        };
        let [a, b, c, d] = self.address.octets();
        Ipv4Addr::new(a, b, c, d.wrapping_add(1 + slot as u8))
    }

    /// A BOOTREPLY to `request` that gives `offered`.
    fn reply(&self, request: &[u8], reply_type: u8, offered: Ipv4Addr) -> Vec<u8> {
        let server: [u8; 4] = self.address.octets();
        let mut reply: Vec<u8> = vec![0; FIXED_BYTES];
        reply[0] = 2;
        reply[1..3].copy_from_slice(&request[1..3]);
        // Transaction id and flags as the client sent them; chaddr likewise.
        reply[4..8].copy_from_slice(&request[4..8]);
        reply[10..12].copy_from_slice(&request[10..12]);
        reply[16..20].copy_from_slice(&offered.octets());
        reply[20..24].copy_from_slice(&server);
        reply[28..44].copy_from_slice(&request[28..44]);
        reply[236..240].copy_from_slice(&MAGIC_COOKIE);
        reply.extend_from_slice(&[OPTION_TYPE, 1, reply_type]);
        for code in [OPTION_SERVER, OPTION_ROUTER, OPTION_DNS] {
            reply.extend_from_slice(&[code, 4]);
            reply.extend_from_slice(&server);
        }
        reply.extend_from_slice(&[OPTION_SUBNET, 4, 255, 255, 255, 0, OPTION_LEASE, 4]);
        reply.extend_from_slice(&LEASE_SECONDS.to_be_bytes());
        reply.push(OPTION_END);
        if reply.len() < MIN_REPLY {
            reply.resize(MIN_REPLY, 0);
        }
        reply
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SERVER: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);

    fn message(kind: u8, mac_end: u8, extra: &[u8]) -> Vec<u8> {
        let mut message: Vec<u8> = vec![0; FIXED_BYTES];
        message[0] = 1;
        message[1] = 1;
        message[2] = 6;
        message[4..8].copy_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        message[10] = 0x80;
        message[28..34].copy_from_slice(&[2, 0, 0, 0, 0, mac_end]);
        message[236..240].copy_from_slice(&MAGIC_COOKIE);
        message.extend_from_slice(&[OPTION_PAD, OPTION_TYPE, 1, kind]);
        message.extend_from_slice(extra);
        message.push(OPTION_END);
        message
    }

    fn option(reply: &[u8], code: u8) -> Option<&[u8]> {
        let mut options: &[u8] = &reply[FIXED_BYTES..];
        while let [found, length, rest @ ..] = options {
            if *found == OPTION_END {
                return None;
            }
            let value: &[u8] = &rest[..usize::from(*length)];
            if *found == code {
                return Some(value);
            }
            options = &rest[usize::from(*length)..];
        }
        None
    }

    #[test]
    fn offers_then_acknowledges_the_same_address() {
        let mut server = DhcpServer::new(SERVER);
        let offer: Vec<u8> = server.answer(&message(DISCOVER, 1, &[])).expect("offer");
        assert_eq!(offer[0], 2);
        assert_eq!(&offer[4..8], &[0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(offer[10], 0x80);
        assert_eq!(&offer[16..20], &[192, 168, 4, 2]);
        assert_eq!(&offer[28..34], &[2, 0, 0, 0, 0, 1]);
        assert!(offer.len() >= MIN_REPLY);
        assert_eq!(option(&offer, OPTION_TYPE), Some(&[OFFER][..]));
        assert_eq!(option(&offer, OPTION_DNS), Some(&[192, 168, 4, 1][..]));
        assert_eq!(option(&offer, OPTION_SUBNET), Some(&[255, 255, 255, 0][..]));

        let ack: Vec<u8> = server
            .answer(&message(REQUEST, 1, &[OPTION_SERVER, 4, 192, 168, 4, 1]))
            .expect("ack");
        assert_eq!(option(&ack, OPTION_TYPE), Some(&[ACK][..]));
        assert_eq!(&ack[16..20], &[192, 168, 4, 2]);
        let second: Vec<u8> = server.answer(&message(DISCOVER, 2, &[])).expect("offer");
        assert_eq!(&second[16..20], &[192, 168, 4, 3]);
    }

    #[test]
    fn ignores_what_is_not_for_it() {
        let mut server = DhcpServer::new(SERVER);
        assert_eq!(
            server.answer(&message(REQUEST, 1, &[OPTION_SERVER, 4, 10, 0, 0, 1])),
            None
        );
        assert_eq!(server.answer(&message(7, 1, &[])), None);
        assert_eq!(server.answer(&[1; 100]), None);
        let mut reply: Vec<u8> = message(DISCOVER, 1, &[]);
        reply[0] = 2;
        assert_eq!(server.answer(&reply), None);
        let mut cut: Vec<u8> = message(DISCOVER, 1, &[]);
        cut.truncate(cut.len() - 2);
        cut.push(OPTION_TYPE);
        assert_eq!(server.answer(&cut), None);
    }

    #[test]
    fn reuses_the_oldest_lease_when_full() {
        let mut server = DhcpServer::new(SERVER);
        for mac_end in 1..=8 {
            assert!(server.answer(&message(DISCOVER, mac_end, &[])).is_some());
        }
        let ninth: Vec<u8> = server.answer(&message(DISCOVER, 9, &[])).expect("offer");
        assert_eq!(&ninth[16..20], &[192, 168, 4, 2]);
        let tenth: Vec<u8> = server.answer(&message(DISCOVER, 10, &[])).expect("offer");
        assert_eq!(&tenth[16..20], &[192, 168, 4, 3]);
    }
}
