//! The blocking HTTP server.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

use lan_http::{Request, Response, Server};

use crate::frame_log::{self, FrameLog};
use crate::page_store::PageStore;
use crate::panel_body::PanelBody;
use crate::query;
use crate::rendered_page::RenderedPage;
use crate::target::Target;

/// Seconds a panel should sleep if asked now, given the page's own poll time in seconds.
pub type SleepPlan = dyn Fn(u32) -> u32 + Send + Sync;

/// Reads a page request's query (everything after `?`) into the target to draw for and the
/// view to draw, or a message for the client when it cannot be read.
pub type Router = dyn Fn(&str) -> Result<(Target, String), String> + Send + Sync;

/// The firmware the EE02 frame runs, offered from [`FrameServer::with_firmware`]'s folder.
pub const FIRMWARE: &str = "ee02-thin";

/// Everything the request handler reads, shared by the connection threads.
struct Shared {
    store: Arc<PageStore>,
    sleep: Arc<SleepPlan>,
    router: Arc<Router>,
    log: Arc<FrameLog>,
    firmware: Option<PathBuf>,
}

/// Serves the pages in a [`PageStore`].
pub struct FrameServer {
    server: Server,
    store: Arc<PageStore>,
    sleep: Arc<SleepPlan>,
    router: Arc<Router>,
    log: Arc<FrameLog>,
    firmware: Option<PathBuf>,
}

impl std::fmt::Debug for FrameServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrameServer")
            .field("server", &self.server)
            .field("store", &self.store)
            .finish_non_exhaustive()
    }
}

impl FrameServer {
    /// Binds to `address`, e.g. `0.0.0.0:8080`.
    ///
    /// # Errors
    ///
    /// Returns an error when the address cannot be bound.
    pub fn bind(address: &str, store: Arc<PageStore>) -> io::Result<Self> {
        let default: Target = store.default_target();
        Ok(Self {
            server: Server::bind(address)?,
            store,
            sleep: Arc::new(|poll_seconds| poll_seconds),
            router: Arc::new(move |query| Ok((Target::from_query(query, default)?, String::new()))),
            log: Arc::new(FrameLog::new()),
            firmware: None,
        })
    }

    /// Reads page requests with `router` instead of [`Target::from_query`] alone, so the app
    /// can offer views (`frame=`, `view=`) and give each frame its own default target.
    #[must_use]
    pub fn with_router(
        mut self,
        router: impl Fn(&str) -> Result<(Target, String), String> + Send + Sync + 'static,
    ) -> Self {
        self.router = Arc::new(router);
        self
    }

    /// Notes each panel's wakes in `log` instead of a log of its own.
    #[must_use]
    pub fn with_frame_log(mut self, log: Arc<FrameLog>) -> Self {
        self.log = log;
        self
    }

    /// Offers firmware from `folder`: `/firmware/<name>.bin` serves a file there, and when
    /// `ee02-thin.bin` and `ee02-thin.version` are both there, `/panel` answers carry
    /// `X-Frame-Firmware` with the version, so frames on another version fetch the file.
    #[must_use]
    pub fn with_firmware(mut self, folder: PathBuf) -> Self {
        self.firmware = Some(folder);
        self
    }

    /// Decides how long panels sleep, e.g. longer at night: `sleep` gets the page's poll time
    /// in seconds and returns the `max-age` to send. By default it is the poll time.
    #[must_use]
    pub fn with_sleep_plan(mut self, sleep: impl Fn(u32) -> u32 + Send + Sync + 'static) -> Self {
        self.sleep = Arc::new(sleep);
        self
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
        let shared = Shared {
            store: Arc::clone(&self.store),
            sleep: Arc::clone(&self.sleep),
            router: Arc::clone(&self.router),
            log: Arc::clone(&self.log),
            firmware: self.firmware.clone(),
        };
        self.server.run(move |request| answer(&shared, request));
    }
}

fn answer(shared: &Shared, request: &Request) -> Response {
    let store: &PageStore = &shared.store;
    let path: &str = request.path();
    if path == "/api/frames" {
        return Response::new(200, shared.log.to_json())
            .with_header("Content-Type", "application/json")
            .with_header("Cache-Control", no_cache());
    }
    if let Some(frame) = path.strip_prefix("/api/frames/") {
        return match shared.log.visit(frame) {
            Some(visit) => Response::new(200, visit.to_json())
                .with_header("Content-Type", "application/json")
                .with_header("Cache-Control", no_cache()),
            None => Response::text(404, "no frame by that name has woken yet"),
        };
    }
    if let Some(file) = path.strip_prefix("/firmware/") {
        return firmware_file(shared.firmware.as_ref(), file);
    }
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
    let (target, view): (Target, String) = match (shared.router)(query) {
        Ok(routed) => routed,
        Err(problem) => return Response::text(400, &problem),
    };
    if path == "/panel" && matches!(target, Target::Canvas(_)) {
        return Response::text(400, "/panel is for panels: ask with panel= and rotation=");
    }
    if !store.is_ready() {
        return Response::text(503, "no page rendered yet");
    }
    if path == "/panel" {
        let battery: Option<u32> = request
            .header("X-Frame-Battery-mV")
            .and_then(|millivolts| millivolts.trim().parse().ok());
        shared.log.record(
            frame_log::frame_name(query::query_value(query, "frame")),
            query::query_value(query, "view")
                .filter(|view| view.len() <= frame_log::MOST_NAME_BYTES)
                .unwrap_or_default(),
            battery,
            request.header("X-Frame-Firmware"),
        );
    }
    let Some(page) = store.view(target, &view) else {
        return Response::text(500, "the page could not be drawn");
    };
    match path {
        "/panel" => match &page.panel {
            Some(body) => {
                let response: Response = panel(request, &page, body, shared.sleep.as_ref());
                match offered_version(shared.firmware.as_ref()) {
                    Some(version) => response.with_header("X-Frame-Firmware", version),
                    None => response,
                }
            }
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
<a href=/panel>/panel</a> (for the panel's firmware) · <a href=/api/frames>/api/frames</a> · views: \
<a href='/preview.png?view=week'>week</a>, <a href='/preview.png?view=rarest'>rarest</a>, \
<a href='/preview.png?view=story'>story</a>, <a href='/preview.png?view=season'>season</a> · other sizes: \
<a href='/preview.png?panel=7.3'>?panel=7.3</a>, \
<a href='/preview.png?w=1920&amp;h=1080'>?w=1920&amp;h=1080</a></p><img src=/preview.png alt='Current page'>";

/// Returns the version of the EE02 firmware on offer, when both its files are in `folder`.
fn offered_version(folder: Option<&PathBuf>) -> Option<String> {
    let folder: &PathBuf = folder?;
    if !folder.join(format!("{FIRMWARE}.bin")).is_file() {
        return None;
    }
    let version: String = fs::read_to_string(folder.join(format!("{FIRMWARE}.version"))).ok()?;
    let version: &str = version.trim();
    let printable: bool = version.bytes().all(|byte| byte.is_ascii_graphic());
    (!version.is_empty() && version.len() <= frame_log::MOST_NAME_BYTES && printable)
        .then(|| String::from(version))
}

/// Serves `/firmware/<file>`: a `.bin` file named in `[a-z0-9-._]` from the firmware folder.
fn firmware_file(folder: Option<&PathBuf>, file: &str) -> Response {
    let Some(folder) = folder else {
        return Response::text(404, "no firmware is offered: set FIRMWARE_DIR");
    };
    let named: bool = file.ends_with(".bin")
        && !file.starts_with('.')
        && file.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-._".contains(&byte)
        });
    if !named || file.contains("..") {
        return Response::text(404, "not found");
    }
    match fs::read(folder.join(file)) {
        Ok(bytes) => Response::new(200, bytes)
            .with_header("Content-Type", "application/octet-stream")
            .with_header("Cache-Control", no_cache()),
        Err(_) => Response::text(404, "not found"),
    }
}

/// Sends the panel's bytes, compressed when the client accepts `deflate`, with how long it
/// should sleep as the time the page stays fresh.
fn panel(request: &Request, page: &RenderedPage, body: &PanelBody, sleep: &SleepPlan) -> Response {
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
    let seconds: u32 = sleep(u32::from(page.poll_minutes) * 60);
    let fresh: String = if seconds == 0 {
        no_cache()
    } else {
        format!("max-age={seconds}")
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
