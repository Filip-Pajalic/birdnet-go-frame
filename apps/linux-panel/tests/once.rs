//! The app end to end against the simulated station, writing pages to a file.

#![allow(clippy::expect_used)] // Tests may panic on failed assumptions.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Runs the app once for `panel` and returns the page it wrote.
fn page(panel: &str, layout: &str) -> Vec<u8> {
    let out: PathBuf = std::env::temp_dir().join(format!(
        "linux-panel-{panel}-{layout}-{}.bin",
        std::process::id()
    ));
    let run: Output = Command::new(env!("CARGO_BIN_EXE_birdnet-go-frame-panel"))
        .arg("--once")
        .env_clear()
        .env("BIRDNET_URL", "mock")
        .env("PANEL", panel)
        .env("LAYOUT", layout)
        .env("PANEL_OUT", &out)
        .output()
        .expect("the app runs");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let bytes: Vec<u8> = fs::read(&out).expect("a page was written");
    fs::remove_file(&out).expect("cleaned up");
    bytes
}

#[test]
fn draws_one_page_for_either_panel() {
    let large: Vec<u8> = page("13.3", "collage");
    assert_eq!(large.len(), 960_000);
    // Six inks, two a byte: every nibble is a controller code.
    assert!(large.iter().all(|&byte| {
        [byte >> 4, byte & 0x0f]
            .iter()
            .all(|&code| code != 4 && code < 7)
    }));
    // Not a blank page: some ink besides white (1) and the paper's dither.
    assert!(large.iter().any(|&byte| byte >> 4 == 0));
    assert_eq!(page("7.3", "list").len(), 192_000);
}
