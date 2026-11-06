//! The frame server over real sockets.

#![allow(clippy::expect_used)] // Tests may panic on setup failures.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use frame_core::{Crc32, IndexedImage, Ink, PanelSize, Resolution, Rgb, Rotation, panel_bytes};
use frame_http::{FrameLog, FrameServer, FrameVisit, PageStore, RenderedPage, Target};
use frame_raster::Canvas;

fn raw_bytes(address: &str, request: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream.write_all(request.as_bytes()).expect("send");
    let mut reply: Vec<u8> = Vec::new();
    stream.read_to_end(&mut reply).expect("read");
    reply
}

fn raw(address: &str, request: &str) -> String {
    String::from_utf8_lossy(&raw_bytes(address, request)).into_owned()
}

/// Splits a raw reply into its head and body.
fn split(reply: &[u8]) -> (String, &[u8]) {
    let end: usize = reply
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("a blank line after the head");
    (
        String::from_utf8_lossy(&reply[..end]).into_owned(),
        &reply[end + 4..],
    )
}

#[test]
fn serves_pages_with_etags() {
    let store: Arc<PageStore> = Arc::new(PageStore::new(Target::Panel {
        panel: PanelSize::Inches13Point3,
        rotation: Rotation::Degrees0,
    }));
    let server = FrameServer::bind("127.0.0.1:0", Arc::clone(&store)).expect("bind");
    let address: String = server.local_addr().expect("address").to_string();
    thread::spawn(move || server.run());
    let base: String = format!("http://{address}");
    let fetch = |path: &str| {
        lan_http::get(&format!("{base}{path}"), Duration::from_secs(5)).expect("fetch")
    };

    assert_eq!(fetch("/healthz").status, 503);
    assert_eq!(fetch("/panel").status, 503);

    let resolution = panel_bytes::page_resolution(PanelSize::Inches13Point3, Rotation::Degrees0);
    let page = IndexedImage::new(resolution, Ink::White);
    let preview = Canvas::new(resolution, Rgb::WHITE);
    let rendered = RenderedPage::new(&page, &preview, Rotation::Degrees0, 15).expect("packs");
    let etag: String = rendered.etag.clone();
    store.set(rendered);

    assert_eq!(fetch("/healthz").body, b"ok");
    let plain = fetch("/panel");
    assert_eq!(plain.status, 200);
    assert_eq!(plain.body.len(), 960_000);
    assert!(
        plain.body.iter().all(|&byte| byte == 0x11),
        "white is code 1"
    );
    let checksum: String = format!("\"{:08x}\"", Crc32::checksum(&plain.body));
    assert_eq!(etag, checksum, "panels check downloads against the ETag");
    assert_eq!(fetch("/frame.e6").status, 404);
    assert_eq!(&fetch("/frame.png").body[1..4], b"PNG");
    assert_eq!(&fetch("/preview.png").body[1..4], b"PNG");
    assert_eq!(fetch("/nope").status, 404);

    let unchanged = raw(
        &address,
        &format!("GET /panel HTTP/1.1\r\nIf-None-Match: {etag}\r\n\r\n"),
    );
    assert!(unchanged.starts_with("HTTP/1.1 304"), "{unchanged}");
    assert!(unchanged.contains(&format!("ETag: {etag}")));
    assert!(
        unchanged.contains("Cache-Control: max-age=900"),
        "{unchanged}"
    );
    let changed = raw(
        &address,
        "GET /panel HTTP/1.1\r\nIf-None-Match: \"0\"\r\n\r\n",
    );
    assert!(changed.starts_with("HTTP/1.1 200"));
    assert!(!changed.contains("Content-Encoding"));

    let reply: Vec<u8> = raw_bytes(
        &address,
        "GET /panel HTTP/1.1\r\nAccept-Encoding: gzip, deflate\r\n\r\n",
    );
    let (head, body) = split(&reply);
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(head.contains("Content-Encoding: deflate"), "{head}");
    assert!(head.contains("Vary: Accept-Encoding"), "{head}");
    assert_eq!(&body[..2], &[0x78, 0x9c], "a zlib stream");
    assert!(body.len() < 10_000, "a blank page is tiny: {}", body.len());
    let compressed_etag: String = format!("{}-z\"", etag.trim_end_matches('"'));
    assert!(head.contains(&format!("ETag: {compressed_etag}")), "{head}");
    let unchanged = raw(
        &address,
        &format!(
            "GET /panel HTTP/1.1\r\nAccept-Encoding: deflate\r\nIf-None-Match: {compressed_etag}\r\n\r\n"
        ),
    );
    assert!(unchanged.starts_with("HTTP/1.1 304"), "{unchanged}");
    assert!(!unchanged.contains("Content-Encoding"), "{unchanged}");
}

#[test]
fn serves_every_size_asked_for() {
    let store: Arc<PageStore> = Arc::new(PageStore::new(Target::Panel {
        panel: PanelSize::Inches13Point3,
        rotation: Rotation::Degrees90,
    }));
    let server = FrameServer::bind("127.0.0.1:0", Arc::clone(&store)).expect("bind");
    let address: String = server.local_addr().expect("address").to_string();
    thread::spawn(move || server.run());
    let fetch = |path: &str| {
        lan_http::get(&format!("http://{address}{path}"), Duration::from_secs(5)).expect("fetch")
    };
    store.publish(Box::new(|target: Target| {
        let resolution = target.resolution();
        let page = IndexedImage::new(resolution, Ink::White);
        let preview = Canvas::new(resolution, Rgb::WHITE);
        RenderedPage::for_target(target, &page, &preview, 15).ok()
    }));

    let portrait = fetch("/panel");
    assert_eq!(portrait.body.len(), 1200 * 1600 / 2);
    let small = fetch("/panel?panel=7.3&rotation=0");
    assert_eq!(small.status, 200);
    assert_eq!(small.body.len(), 800 * 480 / 2);
    let screen = fetch("/preview.png?w=320&h=200");
    assert_eq!(&screen.body[1..4], b"PNG");
    // PNG width and height, big-endian, in the IHDR chunk.
    assert_eq!(&screen.body[16..24], &[0, 0, 1, 64, 0, 0, 0, 200]);
    assert_eq!(fetch("/frame.png?w=320&h=200").status, 200);
    assert_eq!(fetch("/panel?w=320&h=200").status, 400);
    assert_eq!(fetch("/panel?panel=5").status, 400);
    assert_eq!(fetch("/preview.png?w=99999&h=2").status, 400);
}

#[test]
fn a_panel_wakes_draws_and_then_keeps_its_page() {
    use panel_wake::{ResponseHead, SleepLimits, WakeState, page_request};

    let store: Arc<PageStore> = Arc::new(PageStore::new(Target::Panel {
        panel: PanelSize::Inches13Point3,
        rotation: Rotation::Degrees0,
    }));
    let server = FrameServer::bind("127.0.0.1:0", Arc::clone(&store)).expect("bind");
    let address: String = server.local_addr().expect("address").to_string();
    thread::spawn(move || server.run());
    // Stripes of every ink, so the page is not trivially compressible.
    let target = Target::Panel {
        panel: PanelSize::Inches13Point3,
        rotation: Rotation::Degrees90,
    };
    let resolution = target.resolution();
    let pixels: Vec<Ink> = (0..resolution.area())
        .map(|index| Ink::ALL[(index / 7 + index / 1200) % 6])
        .collect();
    let page = IndexedImage::from_pixels(resolution, pixels).expect("fills");
    let preview = Canvas::new(resolution, Rgb::WHITE);
    let expected: Vec<u8> =
        panel_bytes::pack(PanelSize::Inches13Point3, &page, Rotation::Degrees90).expect("packs");
    store.publish(Box::new(move |asked: Target| {
        RenderedPage::for_target(asked, &page, &preview, 30).ok()
    }));

    let mut glass: Vec<u8> = vec![0; panel_bytes::len(PanelSize::Inches13Point3)];
    let mut state = WakeState::default();
    for wake_number in 0..2 {
        let request: String = page_request(
            &address,
            PanelSize::Inches13Point3,
            Some(Rotation::Degrees90),
            state.etag(),
        );
        let reply: Vec<u8> = raw_bytes(&address, &request);
        let (head_text, body) = split(&reply);
        let head = ResponseHead::parse(head_text.as_bytes()).expect("HTTP");
        let wake = state.after_response(&head, body, &mut glass, SleepLimits::default());
        assert_eq!(wake.error, None);
        assert_eq!(wake.sleep_seconds, 30 * 60);
        if wake_number == 0 {
            assert!(wake.show, "the first wake draws");
            assert!(
                head.deflate && body.len() < expected.len() / 4,
                "{}",
                body.len()
            );
            assert_eq!(glass, expected);
        } else {
            assert!(!wake.show, "the second wake keeps the page");
            assert_eq!(head.status, 304);
        }
        state = wake.state;
    }
}

#[test]
fn a_sleep_plan_sets_the_panels_max_age() {
    let store: Arc<PageStore> = Arc::new(PageStore::new(Target::Panel {
        panel: PanelSize::Inches7Point3,
        rotation: Rotation::Degrees0,
    }));
    let server = FrameServer::bind("127.0.0.1:0", Arc::clone(&store))
        .expect("bind")
        .with_sleep_plan(|poll_seconds| poll_seconds * 8);
    let address: String = server.local_addr().expect("address").to_string();
    thread::spawn(move || server.run());
    store.publish(Box::new(|target: Target| {
        let resolution = target.resolution();
        let page = IndexedImage::new(resolution, Ink::White);
        let preview = Canvas::new(resolution, Rgb::WHITE);
        RenderedPage::for_target(target, &page, &preview, 15).ok()
    }));
    let head: String = raw(&address, "GET /panel HTTP/1.1\r\n\r\n");
    assert!(head.contains("Cache-Control: max-age=7200"), "{head}");
}

#[test]
fn frames_get_their_views_log_their_wakes_and_find_firmware() {
    let desk = Target::Panel {
        panel: PanelSize::Inches7Point3,
        rotation: Rotation::Degrees90,
    };
    let store: Arc<PageStore> = Arc::new(PageStore::new(Target::Panel {
        panel: PanelSize::Inches7Point3,
        rotation: Rotation::Degrees0,
    }));
    let folder = std::env::temp_dir().join(format!("frame-http-firmware-{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("a folder");
    std::fs::write(folder.join("ee02-thin.bin"), b"\xe9firmware").expect("written");
    std::fs::write(folder.join("ee02-thin.version"), "0.3.0\n").expect("written");
    let log: Arc<FrameLog> = Arc::new(FrameLog::new());
    let server = FrameServer::bind("127.0.0.1:0", Arc::clone(&store))
        .expect("bind")
        .with_frame_log(Arc::clone(&log))
        .with_firmware(folder.clone())
        .with_router(move |query| {
            let view: String = frame_http::query_value(query, "view")
                .unwrap_or("today")
                .to_owned();
            match frame_http::query_value(query, "frame") {
                Some("desk") => Ok((Target::from_query(query, desk)?, format!("desk/{view}"))),
                _ => Ok((
                    Target::from_query(query, Target::Canvas(Resolution::new(80, 80)))?,
                    view,
                )),
            }
        });
    let address: String = server.local_addr().expect("address").to_string();
    thread::spawn(move || server.run());
    // Each view is painted its own ink, so their pages differ.
    store.publish_views(Box::new(|target: Target, view: &str| {
        let resolution = target.resolution();
        let ink: Ink = if view.ends_with("week") {
            Ink::Red
        } else {
            Ink::White
        };
        let page = IndexedImage::new(resolution, ink);
        let preview = Canvas::new(resolution, Rgb::WHITE);
        RenderedPage::for_target(target, &page, &preview, 15).ok()
    }));

    let today: String = raw(
        &address,
        "GET /panel?frame=desk HTTP/1.1\r\nX-Frame-Battery-mV: 3900\r\nX-Frame-Firmware: 0.2.0\r\n\r\n",
    );
    let week: String = raw(&address, "GET /panel?frame=desk&view=week HTTP/1.1\r\n\r\n");
    let etag = |reply: &str| {
        reply
            .lines()
            .find_map(|line| line.strip_prefix("ETag: "))
            .map(str::to_owned)
    };
    assert!(today.starts_with("HTTP/1.1 200"), "{today}");
    assert_ne!(etag(&today), etag(&week));
    assert!(today.contains("X-Frame-Firmware: 0.3.0"), "{today}");

    let visit: FrameVisit = log.visit("desk").expect("the desk frame woke");
    assert_eq!(visit.view, "week");
    assert_eq!(visit.wakes, 2);
    let frames: String = raw(&address, "GET /api/frames/desk HTTP/1.1\r\n\r\n");
    assert!(frames.contains(r#""frame":"desk""#), "{frames}");

    let reply: Vec<u8> = raw_bytes(&address, "GET /firmware/ee02-thin.bin HTTP/1.1\r\n\r\n");
    let (head, body) = split(&reply);
    assert!(head.contains("Content-Length: 9"), "{head}");
    assert_eq!(body, b"\xe9firmware");
    for refused in ["/firmware/../secret.bin", "/firmware/ee02-thin.version"] {
        let reply: String = raw(&address, &format!("GET {refused} HTTP/1.1\r\n\r\n"));
        assert!(reply.starts_with("HTTP/1.1 404"), "{refused}: {reply}");
    }
    std::fs::remove_dir_all(&folder).ok();
}
