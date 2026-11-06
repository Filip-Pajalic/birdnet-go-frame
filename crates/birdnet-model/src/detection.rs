//! One row of `GET /api/v2/detections/recent`.

use alloc::string::String;
use alloc::vec::Vec;

use crate::json::Json;
use crate::parse_error::ParseError;
use crate::scientific_name::ScientificName;

/// A single detection.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    /// BirdNET-Go's row id.
    pub id: u64,
    /// RFC 3339 time with offset (BirdNET-Go from 2026-03); older versions leave it out.
    pub timestamp: Option<String>,
    /// Local date, `YYYY-MM-DD`.
    pub date: String,
    /// Local wall-clock time, `HH:MM:SS`.
    pub time: String,
    /// Scientific name.
    pub scientific_name: ScientificName,
    /// Common name in BirdNET-Go's configured locale.
    pub common_name: String,
    /// Confidence from 0 to 1.
    pub confidence: f64,
    /// Review state; `false_positive` marks a rejected detection.
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
        let Json::Array(items) = Json::parse(body)? else {
            return Err(ParseError::new("expected a list of detections", 0));
        };
        Ok(items
            .iter()
            .filter_map(Self::from_json)
            .filter(|row| row.is_bird() && !row.is_false_positive())
            .collect())
    }

    /// Reads one row. Rows without a scientific name are skipped; other fields default when
    /// missing or of the wrong type.
    fn from_json(row: &Json) -> Option<Self> {
        let text = |key: &str| String::from(row.str(key).unwrap_or_default());
        Some(Self {
            id: row.number("id").map_or(0, |id| id.max(0.0) as u64),
            timestamp: row.str("timestamp").map(String::from),
            date: text("date"),
            time: text("time"),
            scientific_name: ScientificName::new(row.str("scientificName")?),
            common_name: text("commonName"),
            confidence: row.number("confidence").unwrap_or(0.0),
            verified: row.str("verified").map(String::from),
        })
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
