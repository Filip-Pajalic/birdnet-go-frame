//! The blocking HTTP server.

use std::io;
use std::sync::Arc;

use tiny_http::{Header, Request, Response, Server};

use crate::page_store::PageStore;
use crate::rendered_page::RenderedPage;

/// Serves the pages in a [`PageStore`].
pub struct FrameServer {
    server: Server,
    store: Arc<PageStore>,
}

impl FrameServer {
    /// Binds to `address`, e.g. `0.0.0.0:8080`.
    ///
    /// # Errors
    ///
    /// Returns an error when the address cannot be bound.
    pub fn bind(address: &str, store: Arc<PageStore>) -> io::Result<Self> {
        let server: Server = Server::http(address).map_err(io::Error::other)?;
        Ok(Self { server, store })
    }

    /// Returns the address actually bound, useful when binding port 0 in tests.
    #[must_use]
    pub fn local_port(&self) -> Option<u16> {
        self.server
            .server_addr()
            .to_ip()
            .map(|address| address.port())
    }

    /// Answers requests until the process exits.
    pub fn run(&self) {
        for request in self.server.incoming_requests() {
            if let Err(error) = self.answer(request) {
                eprintln!("frame-http: could not answer a request: {error}");
            }
        }
    }

    fn answer(&self, request: Request) -> io::Result<()> {
        let path: String = request.url().split('?').next().unwrap_or("/").to_owned();
        let page: Option<Arc<RenderedPage>> = self.store.get();
        match (path.as_str(), page) {
            ("/healthz", Some(_)) => request.respond(text(200, "ok")),
            ("/healthz", None) => request.respond(text(503, "no page rendered yet")),
            ("/frame.e6" | "/frame.png" | "/preview.png", None) => {
                request.respond(text(503, "no page rendered yet"))
            }
            ("/frame.e6", Some(page)) => {
                respond_cached(request, &page, &page.e6, "application/octet-stream")
            }
            ("/frame.png", Some(page)) => {
                respond_cached(request, &page, &page.frame_png, "image/png")
            }
            ("/preview.png", Some(page)) => {
                respond_cached(request, &page, &page.preview_png, "image/png")
            }
            ("/", _) => request.respond(html(INDEX)),
            _ => request.respond(text(404, "not found")),
        }
    }
}

const INDEX: &str = "<!doctype html><meta charset=utf-8><title>birdnet-go-frame</title>\
<style>body{font-family:sans-serif;margin:2rem;background:#eee}img{max-width:100%;box-shadow:0 2px 8px #0003}</style>\
<h1>birdnet-go-frame</h1><p><a href=/frame.e6>/frame.e6</a> · <a href=/frame.png>/frame.png</a> · \
<a href=/preview.png>/preview.png</a></p><img src=/preview.png alt='Current page'>";

#[expect(
    clippy::expect_used,
    reason = "every name and value used here is ASCII"
)]
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("header names and values are ASCII")
}

fn text(status: u16, body: &str) -> Response<io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(header("Content-Type", "text/plain"))
}

fn html(body: &str) -> Response<io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(header("Content-Type", "text/html; charset=utf-8"))
}

fn respond_cached(
    request: Request,
    page: &RenderedPage,
    body: &[u8],
    content_type: &str,
) -> io::Result<()> {
    let unchanged: bool = request
        .headers()
        .iter()
        .any(|field| field.field.equiv("If-None-Match") && field.value.as_str() == page.etag);
    if unchanged {
        let response = Response::empty(304).with_header(header("ETag", &page.etag));
        return request.respond(response);
    }
    let response = Response::from_data(body.to_vec())
        .with_header(header("Content-Type", content_type))
        .with_header(header("Cache-Control", "no-cache"))
        .with_header(header("ETag", &page.etag));
    request.respond(response)
}
