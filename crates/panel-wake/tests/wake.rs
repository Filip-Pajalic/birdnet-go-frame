//! Wakes against pages packed and compressed as the server does it.

#![allow(clippy::expect_used)] // Tests may panic on failed assumptions.

use frame_core::{Crc32, PanelSize, inflate_zlib, panel_bytes};
use panel_wake::{ResponseHead, SleepLimits, WakeError, WakeState, page_request};

const LEN: usize = 192_000;

/// A zlib stream of `data` in one stored block, as a stand-in for the server's.
fn stored_zlib(data: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = vec![0x78, 0x01];
    let chunks: Vec<&[u8]> = data.chunks(65_535).collect();
    for (index, chunk) in chunks.iter().enumerate() {
        out.push(u8::from(index + 1 == chunks.len()));
        let length: u16 = chunk.len() as u16;
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&(!length).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    let (mut low, mut high): (u32, u32) = (1, 0);
    for &byte in data {
        low = (low + u32::from(byte)) % 65_521;
        high = (high + low) % 65_521;
    }
    out.extend_from_slice(&((high << 16) | low).to_be_bytes());
    out
}

fn page() -> Vec<u8> {
    (0..LEN)
        .map(|index| [0x11, 0x23, 0x56][index % 3])
        .collect()
}

fn head(status: u16, page: &[u8], deflate: bool) -> ResponseHead {
    let suffix: &str = if deflate { "-z" } else { "" };
    ResponseHead {
        status,
        etag: Some(format!("\"{:08x}{suffix}\"", Crc32::checksum(page))),
        deflate,
        content_length: None,
        max_age: Some(900),
    }
}

#[test]
fn a_new_page_is_unpacked_checked_and_remembered() {
    let sent: Vec<u8> = page();
    let body: Vec<u8> = stored_zlib(&sent);
    let mut buffer: Vec<u8> = vec![0; LEN];
    assert_eq!(inflate_zlib(&body, &mut buffer), Ok(LEN));
    let mut glass: Vec<u8> = vec![0; panel_bytes::len(PanelSize::Inches7Point3)];
    let wake = WakeState::default().after_response(
        &head(200, &sent, true),
        &body,
        &mut glass,
        SleepLimits::default(),
    );
    assert!(wake.show && wake.error.is_none());
    assert_eq!(glass, sent);
    assert_eq!(wake.sleep_seconds, 900);
    let etag: &str = wake.state.etag().expect("remembered");
    assert!(etag.ends_with("-z\""));

    // Through RTC memory, then the next request carries it.
    let kept = WakeState::from_bytes(&wake.state.to_bytes());
    assert_eq!(kept, wake.state);
    let request: String = page_request("frame", PanelSize::Inches7Point3, None, kept.etag());
    assert!(request.contains(&format!("If-None-Match: {etag}\r\n")));

    // Unchanged: keep the glass and sleep as told.
    let unchanged = kept.after_response(
        &ResponseHead {
            status: 304,
            max_age: Some(3600),
            ..ResponseHead::default()
        },
        &[],
        &mut glass,
        SleepLimits::default(),
    );
    assert!(!unchanged.show);
    assert_eq!(unchanged.sleep_seconds, 3600);
    assert_eq!(unchanged.state.etag(), Some(etag));
}

#[test]
fn plain_pages_work_too() {
    let sent: Vec<u8> = page();
    let mut glass: Vec<u8> = vec![0; LEN];
    let wake = WakeState::default().after_response(
        &head(200, &sent, false),
        &sent,
        &mut glass,
        SleepLimits::default(),
    );
    assert!(wake.show);
    assert_eq!(glass, sent);
}

#[test]
fn damaged_or_mismatched_pages_are_not_shown() {
    let sent: Vec<u8> = page();
    let limits = SleepLimits::default();
    let mut glass: Vec<u8> = vec![0; LEN];
    let before = WakeState::default().with_etag(Some("\"00000001\""));

    let mut damaged: Vec<u8> = sent.clone();
    damaged[1000] ^= 0x10;
    let wake = before.after_response(&head(200, &sent, false), &damaged, &mut glass, limits);
    assert_eq!(wake.error, Some(WakeError::Damaged));
    assert!(!wake.show);
    assert_eq!(
        wake.state.etag(),
        Some("\"00000001\""),
        "the glass keeps its page"
    );

    let big: Vec<u8> = vec![0x11; 960_000];
    let wake = before.after_response(
        &head(200, &big, true),
        &stored_zlib(&big),
        &mut glass,
        limits,
    );
    assert!(matches!(wake.error, Some(WakeError::WrongLength { .. })));

    let mut corrupt: Vec<u8> = stored_zlib(&sent);
    let last: usize = corrupt.len() - 1;
    corrupt[last] ^= 1;
    let wake = before.after_response(&head(200, &sent, true), &corrupt, &mut glass, limits);
    assert!(matches!(wake.error, Some(WakeError::Inflate(_))));
}

#[test]
fn failures_back_off_and_a_success_resets() {
    let limits = SleepLimits::default();
    let mut state = WakeState::default();
    let mut sleeps: Vec<u32> = Vec::new();
    for _ in 0..10 {
        let wake = state.after_network_failure(limits);
        sleeps.push(wake.sleep_seconds);
        state = wake.state;
    }
    assert_eq!(&sleeps[..5], &[300, 600, 1200, 2400, 4800]);
    assert_eq!(sleeps[9], 12 * 60 * 60, "capped");
    assert_eq!(state.failures(), 10);
    let starting = state.after_response(
        &ResponseHead {
            status: 503,
            ..ResponseHead::default()
        },
        &[],
        &mut [],
        limits,
    );
    assert_eq!(starting.error, Some(WakeError::Status(503)));
    let back = starting.state.after_response(
        &ResponseHead {
            status: 304,
            ..ResponseHead::default()
        },
        &[],
        &mut [],
        limits,
    );
    assert_eq!(back.state.failures(), 0);
    assert_eq!(back.sleep_seconds, 900, "the default when not told");
}

#[test]
fn rtc_memory_with_rubbish_starts_over() {
    let mut bytes: [u8; WakeState::BYTES] = [0xff; WakeState::BYTES];
    assert_eq!(WakeState::from_bytes(&bytes), WakeState::default());
    bytes[32] = 0;
    assert_eq!(WakeState::from_bytes(&bytes).etag(), None);
}
