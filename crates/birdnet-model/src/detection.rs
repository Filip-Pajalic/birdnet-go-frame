//! One row of `GET /api/v2/detections/recent`.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;

use crate::parse_error::ParseError;
use crate::scientific_name::ScientificName;

/// A single detection.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// BirdNET-Go's row id.
    #[serde(default)]
    pub id: u64,
    /// RFC 3339 time with offset (BirdNET-Go from 2026-03); older versions leave it out.
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Local date, `YYYY-MM-DD`.
    #[serde(default)]
    pub date: String,
    /// Local wall-clock time, `HH:MM:SS`.
    #[serde(default)]
    pub time: String,
    /// Scientific name.
    pub scientific_name: ScientificName,
    /// Common name in BirdNET-Go's configured locale.
    #[serde(default)]
    pub common_name: String,
    /// Confidence from 0 to 1.
    #[serde(default)]
    pub confidence: f64,
    /// Review state; `false_positive` marks a rejected detection.
    #[serde(default)]
    pub verified: Option<String>,
}

impl Detection {
    /// Parses a recent-detections response and keeps only birds that were not marked as false
    /// positives, newest first as BirdNET-Go sends them.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] when the body is not a JSON array of detections.
    pub fn parse_list(body: &[u8]) -> Result<Vec<Self>, ParseError> {
        let mut rows: Vec<Self> = serde_json::from_slice(body).map_err(ParseError::new)?;
        rows.retain(|row| row.is_bird() && !row.is_false_positive());
        Ok(rows)
    }

    /// Returns whether a reviewer marked this detection as wrong.
    #[must_use]
    pub fn is_false_positive(&self) -> bool {
        self.verified.as_deref() == Some("false_positive")
    }

    /// Returns whether the row is a bird rather than one of BirdNET's non-animal labels.
    #[must_use]
    pub fn is_bird(&self) -> bool {
        self.scientific_name.is_binomial() && self.scientific_name.as_str() != self.common_name
    }
}
