//! A small threaded server.

use std::io::{self, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::head;
use crate::request::Request;
use crate::response::Response;

/// Most connections answered at once; more are closed straight away.
const MAX_CONNECTIONS: usize = 32;

/// How long a client may take to send its request, or to take the response.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Answers GET and HEAD requests, one thread per connection, one request per connection.
#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Binds to `address`, e.g. `0.0.0.0:8080` or `127.0.0.1:0` for any free port.
    ///
    /// # Errors
    ///
    /// Returns an error when the address cannot be bound.
    pub fn bind(address: &str) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(address)?,
        })
    }

    /// Returns the address actually bound.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket has no local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    /// Answers requests with `handler` until the process exits. Other methods than GET and
    /// HEAD get 405; requests that are not HTTP/1.x get 400. HEAD gets GET's head.
    pub fn run<H>(&self, handler: H)
    where
        H: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let handler: Arc<H> = Arc::new(handler);
        let busy: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        for stream in self.listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            if busy.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
                busy.fetch_sub(1, Ordering::SeqCst);
                continue;
            }
            let handler: Arc<H> = Arc::clone(&handler);
            let busy: Arc<AtomicUsize> = Arc::clone(&busy);
            thread::spawn(move || {
                if let Err(error) = answer(stream, handler.as_ref()) {
                    if error.kind() != io::ErrorKind::WouldBlock {
                        eprintln!("lan-http: {error}");
                    }
                }
                busy.fetch_sub(1, Ordering::SeqCst);
            });
        }
    }
}

fn answer(mut stream: TcpStream, handler: &impl Fn(&Request) -> Response) -> io::Result<()> {
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let Some((head, _body)) = head::read_head(&mut stream)? else {
        return Ok(());
    };
    let (response, head_only): (Response, bool) = match Request::from_head(head) {
        None => (Response::text(400, "bad request"), false),
        Some(request) => match request.method() {
            "GET" => (handler(&request), false),
            "HEAD" => (handler(&request), true),
            _ => (
                Response::text(405, "only GET and HEAD").with_header("Allow", "GET, HEAD"),
                false,
            ),
        },
    };
    stream.write_all(&response.to_bytes(head_only))?;
    stream.flush()
}
