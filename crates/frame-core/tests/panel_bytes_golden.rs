#![allow(clippy::expect_used)] // Tests may panic on failed assumptions (Google Rust Style Guide).

//! Byte-compatibility with the payload of fugleramme's `/frame.e6`.
//!
//! The expected bytes come from an independent Python reference that packs pages with PIL's
//! transposes exactly as fugleramme's `e6.py` does, over the same test pattern as below.

use frame_core::{IndexedImage, Ink, PanelSize, Resolution, Rotation, panel_bytes};

const BIG: PanelSize = PanelSize::Inches13Point3;

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

fn check(rotation: Rotation, head: &str, tail: &str) {
    let page = pattern(panel_bytes::page_resolution(BIG, rotation));
    let packed: Vec<u8> = panel_bytes::pack(BIG, &page, rotation).expect("page fits the panel");
    assert_eq!(packed.len(), 960_000);
    assert_eq!(packed.len(), panel_bytes::len(BIG));
    assert_eq!(hex(&packed[..32]), head, "first bytes for {rotation:?}");
    assert_eq!(
        hex(&packed[packed.len() - 16..]),
        tail,
        "last bytes for {rotation:?}"
    );
}

#[test]
fn landscape() {
    check(
        Rotation::Degrees0,
        "6532106532106532106532106532106532106532106532106532106532106532",
        "23333355555666660000011111222223",
    );
}

#[test]
fn portrait_quarter_turn() {
    check(
        Rotation::Degrees90,
        "0123560123560123560123560123560123560123560123560123560123560123",
        "33322222111110000066666555553333",
    );
}

#[test]
fn upside_down() {
    check(
        Rotation::Degrees180,
        "3222221111100000666665555533333222221111100000666665555533333222",
        "56012356012356012356012356012356",
    );
}

#[test]
fn portrait_three_quarter_turn() {
    check(
        Rotation::Degrees270,
        "3333555556666600000111112222233333555556666600000111112222233333",
        "10653210653210653210653210653210",
    );
}

#[test]
fn rejects_a_page_of_the_wrong_size() {
    let page = IndexedImage::new(Resolution::new(800, 480), Ink::White);
    assert!(panel_bytes::pack(BIG, &page, Rotation::Degrees0).is_err());
}

#[test]
fn seven_inch_pages_are_native_rows_for_one_controller() {
    let panel = PanelSize::Inches7Point3;
    assert_eq!(
        panel_bytes::page_resolution(panel, Rotation::Degrees90),
        Resolution::new(480, 800)
    );
    let page = pattern(Resolution::new(800, 480));
    let packed: Vec<u8> = panel_bytes::pack(panel, &page, Rotation::Degrees0).expect("fits");
    assert_eq!(packed.len(), 800 * 480 / 2);
    assert_eq!(packed.len(), panel_bytes::len(panel));
    // The first byte is the first two pixels of the top row, left one high.
    let first: u8 = (page.pixels()[0].controller_code() << 4) | page.pixels()[1].controller_code();
    assert_eq!(packed[0], first);
    // The second row starts 400 bytes in: no split between controllers.
    let row: u8 =
        (page.pixels()[800].controller_code() << 4) | page.pixels()[801].controller_code();
    assert_eq!(packed[400], row);
    assert!(panel_bytes::pack(panel, &page, Rotation::Degrees90).is_err());
}
