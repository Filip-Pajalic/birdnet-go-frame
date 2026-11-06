//! One plain HTTP request over TCP.

use alloc::vec;
use alloc::vec::Vec;

use embassy_net::Stack;
use embassy_net::tcp::TcpSocket;
use embassy_time::Duration;
use panel_wake::{FrameAddress, ResponseHead};

use crate::transport_error::TransportError;

/// Bytes of TCP receive window. A larger window keeps more of the page in flight.
const RECEIVE_WINDOW: usize = 16 * 1024;
/// Bytes of send buffer; the request is a few hundred bytes.
const SEND_BUFFER: usize = 1024;
/// Bytes the response grows by for each read.
const READ_CHUNK: usize = 4 * 1024;

/// Waits up to `timeout` for the stack to have an address, from DHCP or a static
/// configuration.
///
/// # Errors
///
/// Returns [`TransportError::Address`] when no address comes in time.
pub async fn wait_for_address(stack: Stack<'_>, timeout: Duration) -> Result<(), TransportError> {
    embassy_time::with_timeout(timeout, stack.wait_config_up())
        .await
        .map_err(|_| TransportError::Address)
}

/// Sends `request` to `frame` and reads the whole response into `response`, which it clears
/// first.
///
/// The response ends at its `Content-Length` or when the server closes the connection, which a
/// request with `Connection: close` asks for. `idle` limits how long the connection may stall,
/// and `limit` how many bytes the response may take.
///
/// # Errors
///
/// Returns [`TransportError::Connect`] when the frame does not accept the connection,
/// [`TransportError::Io`] when it breaks or stalls, or [`TransportError::TooLong`] when the
/// response passes `limit`.
pub async fn get(
    stack: Stack<'_>,
    frame: FrameAddress,
    request: &[u8],
    response: &mut Vec<u8>,
    limit: usize,
    idle: Duration,
) -> Result<(), TransportError> {
    let mut receive: Vec<u8> = vec![0; RECEIVE_WINDOW];
    let mut send: Vec<u8> = vec![0; SEND_BUFFER];
    let mut socket = TcpSocket::new(stack, &mut receive, &mut send);
    socket.set_timeout(Some(idle));
    socket
        .connect((frame.ip(), frame.port()))
        .await
        .map_err(|_| TransportError::Connect)?;
    let mut sent: usize = 0;
    while sent < request.len() {
        sent += socket
            .write(&request[sent..])
            .await
            .map_err(|_| TransportError::Io)?;
    }
    socket.flush().await.map_err(|_| TransportError::Io)?;

    response.clear();
    // The response's full length, once its head has arrived and named one.
    let mut expected: Option<usize> = None;
    loop {
        if expected.is_none() {
            expected = ResponseHead::split(response).and_then(|(head, body)| {
                head.content_length
                    .map(|length| response.len() - body.len() + length)
            });
        }
        if expected.is_some_and(|expected| response.len() >= expected) {
            break;
        }
        let start: usize = response.len();
        if start >= limit {
            socket.abort();
            return Err(TransportError::TooLong);
        }
        response.resize((start + READ_CHUNK).min(limit), 0);
        let read: usize = socket
            .read(&mut response[start..])
            .await
            .map_err(|_| TransportError::Io)?;
        response.truncate(start + read);
        if read == 0 {
            break;
        }
    }
    socket.close();
    Ok(())
}
