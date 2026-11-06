//! The blocking HTTP server.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use lan_http::{Request, Response, Server};

use crate::page_store::PageStore;
use crate::panel_body::PanelBody;
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
    if !matches!(path, "/panel" | "/frame.png" | "/preview.png") {
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
    if path == "/panel" && matches!(target, Target::Canvas(_)) {
        return Response::text(400, "/panel is for panels: ask with panel= and rotation=");
    }
    if !store.is_ready() {
        return Response::text(503, "no page rendered yet");
    }
    let Some(page) = store.page(target) else {
        return Response::text(500, "the page could not be drawn");
    };
    match path {
        "/panel" => match &page.panel {
            Some(body) => panel(request, &page, body),
            None => Response::text(500, "the page could not be packed"),
        },
        "/frame.png" => cached(
            request,
            &page.etag,
            &page.frame_png,
            "image/png",
            no_cache(),
        ),
        _ => cached(
            request,
            &page.etag,
            &page.preview_png,
            "image/png",
            no_cache(),
        ),
    }
}

const INDEX: &str = "<!doctype html><meta charset=utf-8><title>birdnet-go-frame</title>\
<style>body{font-family:sans-serif;margin:2rem;background:#eee}img{max-width:100%;box-shadow:0 2px 8px #0003}</style>\
<h1>birdnet-go-frame</h1><p><a href=/frame.png>/frame.png</a> · <a href=/preview.png>/preview.png</a> · \
<a href=/panel>/panel</a> (for the panel's firmware) · other sizes: \
<a href='/preview.png?panel=7.3'>?panel=7.3</a>, \
<a href='/preview.png?w=1920&amp;h=1080'>?w=1920&amp;h=1080</a></p><img src=/preview.png alt='Current page'>";

/// Sends the panel's bytes, compressed when the client accepts `deflate`, with how long it
/// should sleep as the time the page stays fresh.
fn panel(request: &Request, page: &RenderedPage, body: &PanelBody) -> Response {
    let compressed: bool = request
        .header("Accept-Encoding")
        .is_some_and(accepts_deflate);
    // Each form gets its own ETag, as HTTP asks of the encodings of one resource.
    let (bytes, etag): (&[u8], String) = if compressed {
        let tag: &str = page.etag.trim_end_matches('"');
        (&body.zlib, format!("{tag}-z\""))
    } else {
        (&body.plain, page.etag.clone())
    };
    let fresh: String = if page.poll_minutes == 0 {
        no_cache()
    } else {
        format!("max-age={}", u32::from(page.poll_minutes) * 60)
    };
    let response: Response = cached(request, &etag, bytes, "application/octet-stream", fresh)
        .with_header("Vary", "Accept-Encoding");
    if compressed && response.status() == 200 {
        response.with_header("Content-Encoding", "deflate")
    } else {
        response
    }
}

/// Browsers ask again every time, and get a 304 while the page is unchanged.
fn no_cache() -> String {
    String::from("no-cache")
}

/// Whether an `Accept-Encoding` value allows `deflate`, the zlib format (RFC 9110, 8.4.1.2).
fn accepts_deflate(accepted: &str) -> bool {
    accepted.split(',').any(|item| {
        let mut parts = item.split(';').map(str::trim);
        let coding: &str = parts.next().unwrap_or("");
        let refused: bool = parts.any(|parameter| {
            parameter
                .strip_prefix("q=")
                .is_some_and(|weight| weight.parse::<f32>().is_ok_and(|weight| weight <= 0.0))
        });
        (coding.eq_ignore_ascii_case("deflate") || coding == "*") && !refused
    })
}

/// Answers 304 when the client already has `etag`, otherwise 200 with `body`. `freshness` is the
/// `Cache-Control` value.
fn cached(
    request: &Request,
    etag: &str,
    body: &[u8],
    content_type: &str,
    freshness: String,
) -> Response {
    let response: Response = if request.header("If-None-Match") == Some(etag) {
        Response::new(304, Vec::new())
    } else {
        Response::new(200, body).with_header("Content-Type", content_type.to_owned())
    };
    response
        .with_header("Cache-Control", freshness)
        .with_header("ETag", etag.to_owned())
}

#[cfg(test)]
mod test {
    use super::accepts_deflate;

    #[test]
    fn reads_accept_encoding() {
        assert!(accepts_deflate("deflate"));
        assert!(accepts_deflate("gzip, Deflate;q=0.5"));
        assert!(accepts_deflate("*"));
        assert!(!accepts_deflate("gzip, br"));
        assert!(!accepts_deflate("deflate;q=0"));
        assert!(!accepts_deflate("identity"));
    }
}
