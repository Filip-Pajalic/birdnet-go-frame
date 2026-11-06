//! The frame server over real sockets.

#![allow(clippy::expect_used)] // Tests may panic on setup failures.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use frame_core::{IndexedImage, Ink, PanelSize, Rgb, Rotation, e6};
use frame_http::{FrameServer, PageStore, RenderedPage, Target};
use frame_raster::Canvas;

fn raw(address: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream.write_all(request.as_bytes()).expect("send");
    let mut reply: Vec<u8> = Vec::new();
    stream.read_to_end(&mut reply).expect("read");
    String::from_utf8_lossy(&reply).into_owned()
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
    assert_eq!(fetch("/frame.e6").status, 503);

    let resolution = e6::page_resolution(Rotation::Degrees0);
    let page = IndexedImage::new(resolution, Ink::White);
    let preview = Canvas::new(resolution, Rgb::WHITE);
    let rendered = RenderedPage::new(&page, &preview, Rotation::Degrees0, 15).expect("packs");
    let etag: String = rendered.etag.clone();
    store.set(rendered);

    assert_eq!(fetch("/healthz").body, b"ok");
    let frame = fetch("/frame.e6");
    assert_eq!(frame.status, 200);
    assert_eq!(&frame.body[..4], b"E6F1");
    assert_eq!(frame.body.len(), 20 + 960_000);
    assert_eq!(&fetch("/frame.png").body[1..4], b"PNG");
    assert_eq!(&fetch("/preview.png").body[1..4], b"PNG");
    assert_eq!(fetch("/nope").status, 404);

    let unchanged = raw(
        &address,
        &format!("GET /frame.e6 HTTP/1.1\r\nIf-None-Match: {etag}\r\n\r\n"),
    );
    assert!(unchanged.starts_with("HTTP/1.1 304"), "{unchanged}");
    assert!(unchanged.contains(&format!("ETag: {etag}")));
    let changed = raw(
        &address,
        "GET /frame.e6 HTTP/1.1\r\nIf-None-Match: \"0\"\r\n\r\n",
    );
    assert!(changed.starts_with("HTTP/1.1 200"));
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

    let portrait = fetch("/frame.e6");
    assert_eq!(
        &portrait.body[8..12],
        &[0xb0, 0x04, 0x40, 0x06],
        "1200 x 1600 on the glass"
    );
    let small = fetch("/frame.e6?panel=7.3&rotation=0");
    assert_eq!(small.status, 200);
    assert_eq!(small.body.len(), 20 + 800 * 480 / 2);
    let screen = fetch("/preview.png?w=320&h=200");
    assert_eq!(&screen.body[1..4], b"PNG");
    // PNG width and height, big-endian, in the IHDR chunk.
    assert_eq!(&screen.body[16..24], &[0, 0, 1, 64, 0, 0, 0, 200]);
    assert_eq!(fetch("/frame.png?w=320&h=200").status, 200);
    assert_eq!(fetch("/frame.e6?w=320&h=200").status, 400);
    assert_eq!(fetch("/frame.e6?panel=5").status, 400);
    assert_eq!(fetch("/preview.png?w=99999&h=2").status, 400);
}
