//! Answering requests: UDP datagrams and one HTTP connection at a time.

use alloc::vec;
use alloc::vec::Vec;

use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpMetadata, UdpSocket};
use embassy_net::{IpAddress, IpEndpoint, Ipv4Address, Stack};
use embassy_time::Duration;

/// Bytes of the largest datagram kept: a DHCP or DNS message.
const DATAGRAM: usize = 600;
/// Datagrams queued each way.
const QUEUE: usize = 4;
/// Bytes of the HTTP connection's buffers each way.
const TCP_BUFFER: usize = 4 * 1024;
/// Bytes the request grows by for each read.
const READ_CHUNK: usize = 1024;

/// Where a UDP answer goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyTo {
    /// Back to the sender's address and port, as DNS does.
    Sender,
    /// To everyone on the network, on this port, as a DHCP server does for a device that has no
    /// address yet.
    Broadcast(u16),
}

/// What to do with an HTTP request read so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The request is not complete yet: read on.
    ReadMore,
    /// Send this and close the connection.
    Reply(Vec<u8>),
    /// Send this, close the connection, and stop serving.
    ReplyAndStop(Vec<u8>),
}

/// Answers every datagram to `port` with what `answer` returns for it, until the future is
/// dropped.
pub async fn serve_udp(
    stack: Stack<'_>,
    port: u16,
    reply_to: ReplyTo,
    mut answer: impl FnMut(&[u8]) -> Option<Vec<u8>>,
) -> ! {
    let mut receive_meta: [PacketMetadata; QUEUE] = [PacketMetadata::EMPTY; QUEUE];
    let mut send_meta: [PacketMetadata; QUEUE] = [PacketMetadata::EMPTY; QUEUE];
    let mut receive: Vec<u8> = vec![0; DATAGRAM * QUEUE];
    let mut send: Vec<u8> = vec![0; DATAGRAM * QUEUE];
    let mut socket = UdpSocket::new(
        stack,
        &mut receive_meta,
        &mut receive,
        &mut send_meta,
        &mut send,
    );
    let mut datagram: Vec<u8> = vec![0; DATAGRAM];
    if socket.bind(port).is_err() {
        // Only a second socket on the port can fail to bind; the caller never makes one.
        core::future::pending::<()>().await;
    }
    loop {
        let Ok((length, from)): Result<(usize, UdpMetadata), _> =
            socket.recv_from(&mut datagram).await
        else {
            continue;
        };
        let Some(reply) = answer(&datagram[..length]) else {
            continue;
        };
        let to: IpEndpoint = match reply_to {
            ReplyTo::Sender => from.endpoint,
            ReplyTo::Broadcast(port) => {
                IpEndpoint::new(IpAddress::Ipv4(Ipv4Address::BROADCAST), port)
            }
        };
        // A lost answer is asked for again.
        let _ = socket.send_to(&reply, to).await;
    }
}

/// Serves HTTP on `port`, one connection at a time, until `answer` says to stop.
///
/// Each connection's request is read until `answer` returns something other than
/// [`Answer::ReadMore`] or it passes `limit` bytes; a connection that stalls for `idle` is
/// dropped.
pub async fn serve_http(
    stack: Stack<'_>,
    port: u16,
    limit: usize,
    idle: Duration,
    mut answer: impl FnMut(&[u8]) -> Answer,
) {
    let mut receive: Vec<u8> = vec![0; TCP_BUFFER];
    let mut send: Vec<u8> = vec![0; TCP_BUFFER];
    let mut request: Vec<u8> = Vec::new();
    loop {
        let mut socket = TcpSocket::new(stack, &mut receive, &mut send);
        socket.set_timeout(Some(idle));
        if socket.accept(port).await.is_err() {
            continue;
        }
        request.clear();
        let reply: Option<(Vec<u8>, bool)> = loop {
            match answer(&request) {
                Answer::Reply(bytes) => break Some((bytes, false)),
                Answer::ReplyAndStop(bytes) => break Some((bytes, true)),
                Answer::ReadMore => {}
            }
            let start: usize = request.len();
            if start >= limit {
                break None;
            }
            request.resize((start + READ_CHUNK).min(limit), 0);
            match socket.read(&mut request[start..]).await {
                Ok(0) | Err(_) => break None,
                Ok(read) => request.truncate(start + read),
            }
        };
        let Some((bytes, stop)) = reply else {
            socket.abort();
            continue;
        };
        let mut sent: usize = 0;
        while sent < bytes.len() {
            match socket.write(&bytes[sent..]).await {
                Ok(0) | Err(_) => break,
                Ok(written) => sent += written,
            }
        }
        let _ = socket.flush().await;
        socket.close();
        // Let the close reach the browser before the socket goes.
        let _ = embassy_time::with_timeout(Duration::from_secs(1), socket.flush()).await;
        if stop {
            return;
        }
    }
}
