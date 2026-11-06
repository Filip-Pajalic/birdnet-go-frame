#![allow(clippy::expect_used)] // Tests may panic on failed assumptions (Google Rust Style Guide).

//! Parsing recorded BirdNET-Go responses.

use birdnet_model::{Detection, ScientificName, SpeciesSummary};

#[test]
fn summary_keeps_birds_merges_spellings_and_ranks_by_count() {
    let body: &[u8] = include_bytes!("fixtures/species_summary.json");
    let rows: Vec<SpeciesSummary> = SpeciesSummary::parse_list(body).expect("fixture parses");
    let names: Vec<&str> = rows.iter().map(|row| row.common_name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Great Tit",
            "European Robin",
            "Eurasian Blackbird",
            "Eurasian Blue Tit"
        ]
    );

    let great_tit: &SpeciesSummary = &rows[0];
    assert_eq!(
        great_tit.scientific_name,
        ScientificName::new("Parus major")
    );
    assert_eq!(great_tit.count, 45);
    assert_eq!(great_tit.max_confidence, Some(0.99));
    assert_eq!(
        great_tit.first_heard.as_deref(),
        Some("2026-11-06T04:59:00+01:00")
    );
    assert_eq!(
        great_tit.last_heard.as_deref(),
        Some("2026-11-06T07:45:00+01:00")
    );
}

#[test]
fn recent_drops_non_birds_and_false_positives() {
    let body: &[u8] = include_bytes!("fixtures/detections_recent.json");
    let rows: Vec<Detection> = Detection::parse_list(body).expect("fixture parses");
    let ids: Vec<u64> = rows.iter().map(|row| row.id).collect();
    assert_eq!(ids, [103, 100]);
    assert_eq!(
        rows[0].timestamp.as_deref(),
        Some("2026-11-06T07:45:00+01:00")
    );
}

#[test]
fn rejects_a_body_that_is_not_a_list() {
    assert!(SpeciesSummary::parse_list(br#"{"error": "unauthorized"}"#).is_err());
}
