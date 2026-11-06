//! Reading `geometry.json`: where the bird is on each plate.

use std::collections::HashMap;

use art_pack::Subject;
use lite_json::Json;

/// The bird box recorded for one plate, and the trimmed size it was measured on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Geometry {
    pub(crate) subject: Subject,
    pub(crate) cut: (usize, usize),
}

/// Reads `geometry.json`, keyed by path under the style folder (e.g. `birds/parus-major.webp`).
/// Entries that are not well formed are skipped, as the file is edited by hand.
pub(crate) fn parse(bytes: &[u8]) -> HashMap<String, Geometry> {
    let Ok(document) = Json::parse(bytes) else {
        return HashMap::new();
    };
    let Some(members) = document.as_object() else {
        return HashMap::new();
    };
    members
        .iter()
        .filter_map(|(path, value)| Some((path.clone(), entry(value)?)))
        .collect()
}

fn entry(value: &Json) -> Option<Geometry> {
    let numbers = |key: &str| -> Option<Vec<f64>> {
        value
            .get(key)?
            .as_array()?
            .iter()
            .map(Json::as_number)
            .collect()
    };
    let corners: Vec<f64> = numbers("box")?;
    let cut: Vec<f64> = numbers("cut")?;
    let ([left, top, right, bottom], [width, height]) = (corners.as_slice(), cut.as_slice()) else {
        return None;
    };
    if *width < 1.0 || *height < 1.0 {
        return None;
    }
    Some(Geometry {
        subject: Subject::new(*left as f32, *top as f32, *right as f32, *bottom as f32)?,
        cut: (*width as usize, *height as usize),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reads_good_entries_and_skips_bad_ones() {
        let records = parse(
            br#"{
                "birds/parus-major.webp": {"box": [0.1, 0.2, 0.9, 0.8], "cut": [400, 300]},
                "birds/broken.webp": {"box": [0.9, 0.2, 0.1, 0.8], "cut": [400, 300]},
                "birds/short.webp": {"box": [0.1], "cut": [400, 300]}
            }"#,
        );
        assert_eq!(records.len(), 1);
        let record = records["birds/parus-major.webp"];
        assert_eq!(record.cut, (400, 300));
        assert_eq!(record.subject.left, 0.1);
        assert!(parse(b"not json").is_empty());
    }
}
