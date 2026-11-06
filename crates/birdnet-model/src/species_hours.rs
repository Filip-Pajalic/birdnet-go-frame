//! One species' detections hour by hour, from `GET /api/v2/analytics/time/hourly/batch`.

use alloc::vec::Vec;

use lite_json::Json;

use crate::parse_error::ParseError;
use crate::scientific_name::ScientificName;

/// How many times one species was heard in each local hour of a day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeciesHours {
    /// Scientific name, as it was asked for.
    pub scientific_name: ScientificName,
    /// Detections in each hour, index 0 for midnight to one.
    pub hours: [u32; 24],
}

impl SpeciesHours {
    /// Parses a batch response: an object keyed by scientific name, each holding a list of
    /// `{"hour": 0..23, "count": n}`. Entries with an hour outside the day are skipped, and the
    /// species keep the order BirdNET-Go sends them in.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] when the body is not a JSON object.
    pub fn parse_batch(body: &[u8]) -> Result<Vec<Self>, ParseError> {
        let document: Json = Json::parse(body).map_err(ParseError::from)?;
        let Some(species) = document.as_object() else {
            return Err(ParseError::new("expected an object keyed by species", 0));
        };
        Ok(species
            .iter()
            .map(|(name, rows)| {
                let mut hours: [u32; 24] = [0; 24];
                for row in rows.as_array().unwrap_or_default() {
                    let (Some(hour), Some(count)) = (row.number("hour"), row.number("count"))
                    else {
                        continue;
                    };
                    if (0.0..24.0).contains(&hour) {
                        hours[hour as usize] = count.clamp(0.0, f64::from(u32::MAX)) as u32;
                    }
                }
                Self {
                    scientific_name: ScientificName::new(name),
                    hours,
                }
            })
            .collect())
    }

    /// Returns the detections over the whole day.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.hours.iter().fold(0, |sum, &count| sum.saturating_add(count))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_each_species_hours() {
        let body = br#"{"Parus major":[{"hour":5,"count":12},{"hour":6,"count":3},{"hour":30,"count":9}],"Pica pica":[]}"#;
        let species: Vec<SpeciesHours> = SpeciesHours::parse_batch(body).expect("parses");
        assert_eq!(species.len(), 2);
        assert_eq!(species[0].scientific_name, ScientificName::new("parus major"));
        assert_eq!((species[0].hours[5], species[0].total()), (12, 15));
        assert_eq!(species[1].total(), 0);
        assert!(SpeciesHours::parse_batch(b"[]").is_err());
    }
}
