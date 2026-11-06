//! A folder of plates, as an [`ArtSource`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use art_pack::{ArtImage, ArtSource};
use frame_core::Rgb;

use crate::geometry::{self, Geometry};
use crate::{halo, plate};

const SECONDS_PER_DAY: u64 = 86_400;

/// A fugleramme-style art folder: `birds/*.webp` and an optional `geometry.json`.
///
/// ```no_run
/// use art_pack::ArtSource;
/// use art_webp::ArtFolder;
///
/// let mut art = ArtFolder::open("art/classic")?;
/// let great_tit = art.art_for("Parus major");
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct ArtFolder {
    birds: PathBuf,
    /// File stems (`parus-major`, `parus-major-2`) found in `birds/`.
    stems: Vec<String>,
    geometry: HashMap<String, Geometry>,
    paper: Rgb,
    max_side: usize,
    day: Option<u64>,
}

impl ArtFolder {
    /// The paper the plates are toned to by default: fugleramme's panel paper, a little darker
    /// than its screen paper so the panel's dither keeps it a clean cream.
    pub const PANEL_PAPER: Rgb = Rgb::new(233, 225, 202);

    /// Plates are shrunk to at most this many pixels on their longest side by default, enough
    /// for the largest bird on a 1600-pixel page.
    pub const MAX_SIDE: usize = 800;

    /// Opens the style folder at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when `path/birds` cannot be listed. A missing or broken
    /// `geometry.json` is not an error: plates are then sized whole.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let birds: PathBuf = path.as_ref().join("birds");
        let mut stems: Vec<String> = Vec::new();
        for entry in fs::read_dir(&birds)? {
            let file: PathBuf = entry?.path();
            if file.extension().and_then(|extension| extension.to_str()) != Some("webp") {
                continue;
            }
            if let Some(stem) = file.file_stem().and_then(|stem| stem.to_str()) {
                stems.push(stem.to_owned());
            }
        }
        stems.sort_unstable();
        let geometry: HashMap<String, Geometry> = fs::read(path.as_ref().join("geometry.json"))
            .map(|bytes| geometry::parse(&bytes))
            .unwrap_or_default();
        Ok(Self {
            birds,
            stems,
            geometry,
            paper: Self::PANEL_PAPER,
            max_side: Self::MAX_SIDE,
            day: None,
        })
    }

    /// Returns the folder toning plates to `paper` instead of [`Self::PANEL_PAPER`].
    #[must_use]
    pub fn with_paper(mut self, paper: Rgb) -> Self {
        self.paper = paper;
        self
    }

    /// Returns the folder shrinking plates to at most `max_side` pixels.
    #[must_use]
    pub fn with_max_side(mut self, max_side: usize) -> Self {
        self.max_side = max_side;
        self
    }

    /// Returns the folder choosing plates for days since 1970 `day` rather than today, so
    /// tests and previews are repeatable.
    #[must_use]
    pub fn with_day(mut self, day: u64) -> Self {
        self.day = Some(day);
        self
    }

    /// Returns how many plates the folder holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.stems.len()
    }

    /// Returns whether the folder holds no plates.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stems.is_empty()
    }

    /// Returns the stems of every plate of the species, the plain one first and then the
    /// numbered ones in order. A hybrid's file (`tetrao-urogallus-x-lagopus-lagopus`) is not a
    /// variant of `tetrao-urogallus`.
    fn variants(&self, key: &str) -> Vec<&str> {
        let mut numbered: Vec<(u32, &str)> = self
            .stems
            .iter()
            .filter_map(|stem| {
                let number: u32 = stem.strip_prefix(key)?.strip_prefix('-')?.parse().ok()?;
                Some((number, stem.as_str()))
            })
            .collect();
        numbered.sort_unstable();
        let plain = self
            .stems
            .iter()
            .find(|stem| *stem == key)
            .map(String::as_str);
        plain
            .into_iter()
            .chain(numbered.into_iter().map(|(_, stem)| stem))
            .collect()
    }

    fn today(&self) -> u64 {
        self.day.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs() / SECONDS_PER_DAY)
                .unwrap_or(0)
        })
    }

    fn load(&self, stem: &str) -> Option<ArtImage> {
        let bytes: Vec<u8> = fs::read(self.birds.join(format!("{stem}.webp"))).ok()?;
        let mut picture: ArtImage = plate::trim(&plate::decode(&bytes)?);
        if let Some(record) = self.geometry.get(&format!("birds/{stem}.webp")) {
            let size = picture.resolution();
            // The box was measured on the trimmed plate; ignore it if the plate has changed.
            if record.cut == (size.width, size.height) {
                picture = picture.with_subject(record.subject);
            }
        }
        Some(halo::blend_into(
            &plate::shrink(&picture, self.max_side),
            self.paper,
        ))
    }
}

impl ArtSource for ArtFolder {
    fn art_for(&mut self, scientific_name: &str) -> Option<ArtImage> {
        let key: String = file_key(scientific_name);
        let variants: Vec<&str> = self.variants(&key);
        if variants.is_empty() {
            return None;
        }
        let seed: u64 = fnv1a(format!("{scientific_name}/{}", self.today()).as_bytes());
        let index: usize = usize::try_from(seed % variants.len() as u64).unwrap_or(0);
        self.load(variants[index])
    }
}

/// Returns the file stem for a scientific name: `Parus major` becomes `parus-major`.
fn file_key(scientific_name: &str) -> String {
    scientific_name
        .split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<String>>()
        .join("-")
}

/// The 64-bit FNV-1a hash: stable across runs and platforms, unlike the standard hasher.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash: u64, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod test {
    use super::*;

    fn folder(stems: &[&str]) -> ArtFolder {
        ArtFolder {
            birds: PathBuf::new(),
            stems: stems.iter().map(|stem| (*stem).to_owned()).collect(),
            geometry: HashMap::new(),
            paper: ArtFolder::PANEL_PAPER,
            max_side: ArtFolder::MAX_SIDE,
            day: Some(0),
        }
    }

    #[test]
    fn finds_variants_in_order_and_skips_hybrids() {
        let art = folder(&[
            "tetrao-urogallus-10",
            "tetrao-urogallus-2",
            "tetrao-urogallus-x-lagopus-lagopus",
            "tetrao-urogallus",
            "tetrao-tetrix",
        ]);
        assert_eq!(
            art.variants("tetrao-urogallus"),
            [
                "tetrao-urogallus",
                "tetrao-urogallus-2",
                "tetrao-urogallus-10"
            ]
        );
        assert!(art.variants("parus-major").is_empty());
    }

    #[test]
    fn makes_file_keys() {
        assert_eq!(file_key("Parus  major"), "parus-major");
        assert_eq!(file_key("Corvus corone cornix"), "corvus-corone-cornix");
    }

    #[test]
    fn hash_is_stable() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn missing_folder_is_an_error() {
        assert!(ArtFolder::open("/nonexistent/style").is_err());
    }
}
