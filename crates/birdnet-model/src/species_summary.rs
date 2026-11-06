//! One row of `GET /api/v2/analytics/species/summary`.

use alloc::string::String;
use alloc::vec::Vec;

use crate::json::Json;
use crate::parse_error::ParseError;
use crate::scientific_name::ScientificName;

/// A species heard in the requested date range, with false positives already excluded by
/// BirdNET-Go.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeciesSummary {
    /// Scientific name.
    pub scientific_name: ScientificName,
    /// Common name in BirdNET-Go's configured locale.
    pub common_name: String,
    /// Number of detections in the range.
    pub count: u32,
    /// Earliest detection, as BirdNET-Go formats it.
    pub first_heard: Option<String>,
    /// Latest detection, as BirdNET-Go formats it.
    pub last_heard: Option<String>,
    /// Highest confidence in the range, from 0 to 1.
    pub max_confidence: Option<f64>,
}

impl SpeciesSummary {
    /// Parses a summary response, keeps only birds, merges rows that name the same species
    /// differently (BirdNET-Go keeps old spellings after a reclassification), and sorts by
    /// count, most detections first.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] when the body is not a JSON array of summary rows.
    pub fn parse_list(body: &[u8]) -> Result<Vec<Self>, ParseError> {
        let Json::Array(items) = Json::parse(body)? else {
            return Err(ParseError::new("expected a list of species", 0));
        };
        let rows = items.iter().filter_map(Self::from_json);
        let mut merged: Vec<Self> = Vec::with_capacity(items.len());
        for row in rows {
            if !row.is_bird() {
                continue;
            }
            match merged
                .iter_mut()
                .find(|held| held.scientific_name == row.scientific_name)
            {
                Some(held) => held.absorb(row),
                None => merged.push(row),
            }
        }
        merged.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.common_name.cmp(&right.common_name))
        });
        Ok(merged)
    }

    /// Reads one row. Rows without a scientific name are skipped; other fields default when
    /// missing or of the wrong type.
    fn from_json(row: &Json) -> Option<Self> {
        Some(Self {
            scientific_name: ScientificName::new(row.str("scientific_name")?),
            common_name: String::from(row.str("common_name").unwrap_or_default()),
            count: row
                .number("count")
                .map_or(0, |count| count.clamp(0.0, f64::from(u32::MAX)) as u32),
            first_heard: row.str("first_heard").map(String::from),
            last_heard: row.str("last_heard").map(String::from),
            max_confidence: row.number("max_confidence"),
        })
    }

    /// Returns whether the row is a bird rather than one of BirdNET's non-animal labels.
    ///
    /// BirdNET labels sounds like `Engine` or `Dog` with the same text as both names; those are
    /// dropped. Bats and frogs have real binomials and are not filtered here.
    #[must_use]
    pub fn is_bird(&self) -> bool {
        self.scientific_name.is_binomial() && self.scientific_name.as_str() != self.common_name
    }

    fn absorb(&mut self, other: Self) {
        self.count = self.count.saturating_add(other.count);
        self.max_confidence = match (self.max_confidence, other.max_confidence) {
            (Some(held), Some(new)) => Some(held.max(new)),
            (held, new) => held.or(new),
        };
        if other.last_heard > self.last_heard {
            self.last_heard = other.last_heard;
        }
        if self.first_heard.is_none()
            || (other.first_heard.is_some() && other.first_heard < self.first_heard)
        {
            self.first_heard = other.first_heard;
        }
    }
}
