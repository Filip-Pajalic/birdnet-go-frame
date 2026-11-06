//! The blocking HTTP server.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use lan_http::{Request, Response, Server};

use crate::page_store::PageStore;
use crate::rendered_page::RenderedPage;
use crate::target::Target;

/// Serves the pages in a [`PageStore`].
#[derive(Debug)]
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
        Ok(Self {
            server: Server::bind(address)?,
            store,
        })
    }

    /// Returns the address actually bound, useful when binding port 0 in tests.
    ///
    /// # Errors
    ///
    /// Returns an error if the socket has no local address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.server.local_addr()
    }

    /// Answers requests until the process exits.
    pub fn run(&self) {
        let store: Arc<PageStore> = Arc::clone(&self.store);
        self.server.run(move |request| answer(&store, request));
    }
}

fn answer(store: &PageStore, request: &Request) -> Response {
    let path: &str = request.path();
    if path == "/" {
        return Response::new(200, INDEX).with_header("Content-Type", "text/html; charset=utf-8");
    }
    if path == "/healthz" {
        return if store.is_ready() {
            Response::text(200, "ok")
        } else {
            Response::text(503, "no page rendered yet")
        };
    }
    if !matches!(path, "/frame.e6" | "/frame.png" | "/preview.png") {
        return Response::text(404, "not found");
    }
    let query: &str = request
        .target()
        .split_once('?')
        .map_or("", |(_, query)| query);
    let target: Target = match Target::from_query(query, store.default_target()) {
        Ok(target) => target,
        Err(problem) => return Response::text(400, &problem),
    };
    if path == "/frame.e6" && matches!(target, Target::Canvas(_)) {
        return Response::text(
            400,
            "/frame.e6 is for panels: ask with panel= and rotation=",
        );
    }
    if !store.is_ready() {
        return Response::text(503, "no page rendered yet");
    }
    let Some(page) = store.page(target) else {
        return Response::text(500, "the page could not be drawn");
    };
    match path {
        "/frame.e6" => match &page.e6 {
            Some(e6) => cached(request, &page, e6, "application/octet-stream"),
            None => Response::text(500, "the page could not be packed"),
        },
        "/frame.png" => cached(request, &page, &page.frame_png, "image/png"),
        _ => cached(request, &page, &page.preview_png, "image/png"),
    }
}

const INDEX: &str = "<!doctype html><meta charset=utf-8><title>birdnet-go-frame</title>\
<style>body{font-family:sans-serif;margin:2rem;background:#eee}img{max-width:100%;box-shadow:0 2px 8px #0003}</style>\
<h1>birdnet-go-frame</h1><p><a href=/frame.e6>/frame.e6</a> · <a href=/frame.png>/frame.png</a> · \
<a href=/preview.png>/preview.png</a> · other sizes: <a href='/preview.png?panel=7.3'>?panel=7.3</a>, \
<a href='/preview.png?w=1920&amp;h=1080'>?w=1920&amp;h=1080</a></p><img src=/preview.png alt='Current page'>";

fn cached(request: &Request, page: &RenderedPage, body: &[u8], content_type: &str) -> Response {
    if request.header("If-None-Match") == Some(page.etag.as_str()) {
        return Response::new(304, Vec::new()).with_header("ETag", page.etag.clone());
    }
    Response::new(200, body)
        .with_header("Content-Type", content_type.to_owned())
        .with_header("Cache-Control", "no-cache")
        .with_header("ETag", page.etag.clone())
}
