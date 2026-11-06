#![allow(clippy::expect_used)] // Tests may panic on failed assumptions (Google Rust Style Guide).

//! Byte-compatibility with fugleramme's `/frame.e6`.
//!
//! The expected bytes come from an independent Python reference that packs pages with PIL's
//! transposes exactly as fugleramme's `e6.py` does, over the same test pattern as below.

use frame_core::{IndexedImage, Ink, Resolution, Rotation, e6};

fn pattern(resolution: Resolution) -> IndexedImage {
    let mut pixels: Vec<Ink> = Vec::with_capacity(resolution.area());
    for y in 0..resolution.height {
        for x in 0..resolution.width {
            let index: usize = (x * 7 + y * 13 + (x * y) % 5) % 6;
            pixels.push(Ink::ALL[index]);
        }
    }
    IndexedImage::from_pixels(resolution, pixels).expect("pattern fills the resolution")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn check(rotation: Rotation, poll_minutes: u8, header: &str, head: &str, tail: &str) {
    let page = pattern(e6::page_resolution(rotation));
    let packed: Vec<u8> = e6::encode(&page, rotation, poll_minutes).expect("page fits the panel");
    assert_eq!(packed.len(), 960_020);
    assert_eq!(hex(&packed[..20]), header, "header for {rotation:?}");
    assert_eq!(
        hex(&packed[20..52]),
        head,
        "first payload bytes for {rotation:?}"
    );
    assert_eq!(
        hex(&packed[packed.len() - 16..]),
        tail,
        "last payload bytes for {rotation:?}"
    );
}

#[test]
fn landscape() {
    check(
        Rotation::Degrees0,
        15,
        "453646310100010fb004400600a60e0009586664",
        "6532106532106532106532106532106532106532106532106532106532106532",
        "23333355555666660000011111222223",
    );
}

#[test]
fn portrait_quarter_turn() {
    check(
        Rotation::Degrees90,
        30,
        "453646310100011eb004400600a60e00de21f540",
        "0123560123560123560123560123560123560123560123560123560123560123",
        "33322222111110000066666555553333",
    );
}

#[test]
fn upside_down() {
    check(
        Rotation::Degrees180,
        0,
        "4536463101000100b004400600a60e00bddfcfff",
        "3222221111100000666665555533333222221111100000666665555533333222",
        "56012356012356012356012356012356",
    );
}

#[test]
fn portrait_three_quarter_turn() {
    check(
        Rotation::Degrees270,
        255,
        "45364631010001ffb004400600a60e00c3adc13c",
        "3333555556666600000111112222233333555556666600000111112222233333",
        "10653210653210653210653210653210",
    );
}

#[test]
fn header_verifies_its_payload() {
    let page = pattern(e6::page_resolution(Rotation::Degrees0));
    let packed: Vec<u8> = e6::encode(&page, Rotation::Degrees0, 5).expect("page fits the panel");
    let header = frame_core::E6Header::parse(&packed).expect("header parses");
    assert_eq!(header.verify_payload(&packed[20..]), Ok(()));
    let mut damaged: Vec<u8> = packed[20..].to_vec();
    damaged[1234] ^= 0x10;
    assert!(header.verify_payload(&damaged).is_err());
}

#[test]
fn rejects_a_page_of_the_wrong_size() {
    let page = IndexedImage::new(Resolution::new(800, 480), Ink::White);
    assert!(e6::encode(&page, Rotation::Degrees0, 5).is_err());
}
