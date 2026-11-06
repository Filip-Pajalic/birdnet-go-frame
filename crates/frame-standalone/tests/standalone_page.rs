//! A standalone frame's whole wake on the host: BirdNET-Go's answer in, the panel's bytes out,
//! drawn the way the ESP32 draws them.

#![allow(clippy::expect_used)] // Tests may panic on failed assumptions.

use art_mock::MockArt;
use art_pack::{ArtBundle, BundleEntry};
use birdnet_model::SpeciesSummary;
use frame_artist::{Artist, Layout, PageStyle};
use frame_core::{PanelSize, Rotation};
use frame_raster::{Canvas, Renderer};
use frame_standalone::{BirdnetAnswer, StandaloneMemory, StandaloneWake};

const FONT: &[u8] = include_bytes!("../../../assets/fonts/ebgaramond/EBGaramond-Italic.ttf");
const ANSWER: &[u8] = b"HTTP/1.1 200 OK\r\nDate: Fri, 06 Nov 2026 07:00:00 GMT\r\n\
    Content-Type: application/json\r\n\r\n[\
    {\"scientific_name\":\"Parus major\",\"common_name\":\"Great Tit\",\"count\":31},\
    {\"scientific_name\":\"Pica pica\",\"common_name\":\"Magpie\",\"count\":12},\
    {\"scientific_name\":\"Erithacus rubecula\",\"common_name\":\"Robin\",\"count\":4}]";

fn style() -> PageStyle {
    PageStyle {
        layout: Layout::Collage,
        max_birds: 12,
        names: true,
        ..PageStyle::default()
    }
}

#[test]
fn draws_both_panels_from_an_answer_and_then_waits_for_news() {
    let answer: BirdnetAnswer<'_> = BirdnetAnswer::parse(ANSWER).expect("HTTP");
    let now: i64 = answer.date.expect("a date");
    let species: Vec<SpeciesSummary> = answer.species().expect("birds");
    let day: i64 = now.div_euclid(86_400);
    let memory = StandaloneMemory::default();
    let wake: StandaloneWake = memory.after_species(day, &species, Layout::Collage, 15);
    assert!(wake.show);
    for (panel, length) in [
        (PanelSize::Inches13Point3, 960_000),
        (PanelSize::Inches7Point3, 192_000),
    ] {
        let renderer = Renderer::new(FONT).expect("the font loads");
        let mut artist = Artist::new(renderer, MockArt::new(200), style());
        artist.start_day(day);
        let size = frame_standalone::drawing_resolution(panel, Rotation::Degrees0);
        let page: Canvas = artist.draw(&species, "", size);
        let bytes: Vec<u8> =
            frame_artist::panel_bytes(page, artist.paper(), panel, Rotation::Degrees0)
                .expect("the page fits");
        assert_eq!(bytes.len(), length);
    }
    let shown: StandaloneMemory = memory.after_shown(wake.fingerprint);
    assert!(!shown.after_species(day, &species, Layout::Collage, 15).show);
}

#[test]
fn draws_from_a_bundle_of_art() {
    let tit = MockArt::new(64).paint("Parus major");
    let artp: Vec<u8> = tit.encode();
    // A zlib stream of one stored block.
    let mut zlib: Vec<u8> = vec![0x78, 0x01, 0x01];
    let len: u16 = u16::try_from(artp.len()).expect("small");
    zlib.extend_from_slice(&len.to_le_bytes());
    zlib.extend_from_slice(&(!len).to_le_bytes());
    zlib.extend_from_slice(&artp);
    zlib.extend_from_slice(&adler32(&artp).to_be_bytes());
    let bundle_bytes: Vec<u8> = ArtBundle::encode([BundleEntry {
        name: "Parus major",
        zlib: &zlib,
        unpacked_len: artp.len(),
    }]);
    let bundle: ArtBundle<'_> = ArtBundle::parse(&bundle_bytes).expect("a bundle");
    let species: Vec<SpeciesSummary> = BirdnetAnswer::parse(ANSWER)
        .expect("HTTP")
        .species()
        .expect("birds");
    let renderer = Renderer::new(FONT).expect("the font loads");
    let mut artist = Artist::new(renderer, bundle, style());
    let size = frame_standalone::drawing_resolution(PanelSize::Inches7Point3, Rotation::Degrees0);
    let page: Canvas = artist.draw(&species, "", size);
    // Only the great tit has a picture; the page is not bare paper.
    assert!(page.pixels().iter().any(|&pixel| pixel != artist.paper()));
}

fn adler32(data: &[u8]) -> u32 {
    let (mut low, mut high): (u32, u32) = (1, 0);
    for &byte in data {
        low = (low + u32::from(byte)) % 65_521;
        high = (high + low) % 65_521;
    }
    (high << 16) | low
}
