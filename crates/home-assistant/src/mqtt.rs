//! Just enough MQTT 3.1.1 to publish: connect, publish retained messages at QoS 0, disconnect.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::message::Message;

/// Seconds the broker may wait between packets before it drops the connection.
const KEEP_ALIVE_SECONDS: u16 = 30;

/// Connects to the broker at `address` (`host:port`) as `client_id`, with `login` when the
/// broker asks for one, publishes `messages` retained, and disconnects.
///
/// # Errors
///
/// Returns an error when the broker cannot be reached, refuses the connection, or the
/// connection breaks.
pub fn publish(
    address: &str,
    client_id: &str,
    login: Option<(&str, &str)>,
    messages: &[Message],
    timeout: Duration,
) -> io::Result<()> {
    let socket = address.to_socket_addrs()?.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "the broker's name did not resolve")
    })?;
    let mut stream: TcpStream = TcpStream::connect_timeout(&socket, timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(&connect_packet(client_id, login))?;
    let mut acknowledgement: [u8; 4] = [0; 4];
    stream.read_exact(&mut acknowledgement)?;
    match acknowledgement {
        [0x20, 0x02, _, 0] => {}
        [0x20, 0x02, _, 4 | 5] => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "the broker refused the login: check MQTT_URL's user and password",
            ));
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the broker refused the connection",
            ));
        }
    }
    for message in messages {
        stream.write_all(&publish_packet(&message.topic, message.payload.as_bytes()))?;
    }
    stream.write_all(&[0xe0, 0x00])?;
    stream.flush()
}

/// Returns a CONNECT packet: protocol level 4, a clean session, and the login if given.
#[must_use]
pub fn connect_packet(client_id: &str, login: Option<(&str, &str)>) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    push_string(&mut body, b"MQTT");
    body.push(4);
    let flags: u8 = match login {
        Some((_, password)) if !password.is_empty() => 0b1100_0010,
        Some(_) => 0b1000_0010,
        None => 0b0000_0010,
    };
    body.push(flags);
    body.extend_from_slice(&KEEP_ALIVE_SECONDS.to_be_bytes());
    push_string(&mut body, client_id.as_bytes());
    if let Some((user, password)) = login {
        push_string(&mut body, user.as_bytes());
        if !password.is_empty() {
            push_string(&mut body, password.as_bytes());
        }
    }
    packet(0x10, &body)
}

/// Returns a PUBLISH packet for `payload` on `topic`, retained, at QoS 0.
#[must_use]
pub fn publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::with_capacity(topic.len() + payload.len() + 2);
    push_string(&mut body, topic.as_bytes());
    body.extend_from_slice(payload);
    packet(0x31, &body)
}

/// Prefixes `body` with the packet type and its remaining length.
fn packet(kind: u8, body: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = vec![kind];
    let mut length: usize = body.len();
    loop {
        let mut byte: u8 = u8::try_from(length % 128).unwrap_or(0);
        length /= 128;
        if length > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if length == 0 {
            break;
        }
    }
    out.extend_from_slice(body);
    out
}

/// Appends `text` as an MQTT string: two bytes of length, then the bytes.
fn push_string(out: &mut Vec<u8>, text: &[u8]) {
    let length: u16 = u16::try_from(text.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(&text[..usize::from(length)]);
}

#[cfg(test)]
mod test {
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    #[test]
    fn encodes_lengths_past_one_byte() {
        let long: Vec<u8> = vec![b'x'; 300];
        let packet: Vec<u8> = publish_packet("t", &long);
        // 3 bytes of topic, 300 of payload: 303 is 0xaf 0x02.
        assert_eq!(&packet[..4], &[0x31, 0xaf, 0x02, 0x00]);
        assert_eq!(packet.len(), 3 + 303);
    }

    #[test]
    fn connects_with_a_login() {
        let packet: Vec<u8> = connect_packet("frame", Some(("user", "pw")));
        assert_eq!(
            packet,
            [
                &[0x10, 27, 0, 4][..],
                b"MQTT",
                &[4, 0b1100_0010, 0, 30, 0, 5],
                b"frame",
                &[0, 4],
                b"user",
                &[0, 2],
                b"pw"
            ]
            .concat()
        );
    }

    #[test]
    fn publishes_to_a_broker() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let address: String = listener.local_addr().expect("address").to_string();
        let broker = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("a client");
            let mut connect: [u8; 2] = [0; 2];
            stream.read_exact(&mut connect).expect("connect head");
            let mut rest: Vec<u8> = vec![0; usize::from(connect[1])];
            stream.read_exact(&mut rest).expect("connect body");
            stream
                .write_all(&[0x20, 0x02, 0x00, 0x00])
                .expect("connack");
            let mut received: Vec<u8> = Vec::new();
            stream.read_to_end(&mut received).expect("the rest");
            received
        });
        let message = Message {
            topic: String::from("a/b"),
            payload: String::from("{}"),
        };
        publish(&address, "test", None, &[message], Duration::from_secs(5)).expect("published");
        let received: Vec<u8> = broker.join().expect("the broker ran");
        assert_eq!(
            received,
            [0x31, 7, 0, 3, b'a', b'/', b'b', b'{', b'}', 0xe0, 0x00]
        );
    }
}
