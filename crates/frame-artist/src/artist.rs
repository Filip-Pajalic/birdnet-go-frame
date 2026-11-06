//! Draws pages: picks the layout, finds each bird's picture and size, renders.

use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use art_pack::{ArtImage, ArtSource};
use bird_sizes::BirdSizes;
use birdnet_model::SpeciesSummary;
use frame_core::{Resolution, Rgb};
use frame_layout::{
    Collage, CollageBird, CollageOptions, PageLayout, Silhouette, TextList, TextListOptions,
    size_weights,
};
use frame_raster::{Canvas, Renderer};
use page_text::PageText;

use crate::layout::Layout;
use crate::page_style::PageStyle;

/// A species' picture and its outline, worked out once a day.
struct Picture {
    art: ArtImage,
    silhouette: Silhouette,
}

/// Draws pages with one font, one source of pictures and one style.
pub struct Artist<A: ArtSource> {
    renderer: Renderer,
    art: A,
    pictures: BTreeMap<String, Option<Picture>>,
    day: Option<i64>,
    sizes: BirdSizes<'static>,
    style: PageStyle,
}

impl<A: ArtSource> Artist<A> {
    /// Creates an artist that draws with `renderer`'s font and paper, `art`'s pictures and
    /// `style`.
    #[must_use]
    pub fn new(renderer: Renderer, art: A, style: PageStyle) -> Self {
        Self {
            renderer,
            art,
            pictures: BTreeMap::new(),
            day: None,
            sizes: BirdSizes::bundled(),
            style,
        }
    }

    /// Returns the paper pages are drawn on.
    #[must_use]
    pub fn paper(&self) -> Rgb {
        self.renderer.paper()
    }

    /// Returns the page's words.
    #[must_use]
    pub fn words(&self) -> &PageText {
        &self.style.words
    }

    /// Starts day `day` (days since 1970): art sources may pick another plate for a species,
    /// so cached pictures are forgotten.
    pub fn start_day(&mut self, day: i64) {
        if self.day != Some(day) {
            self.pictures.clear();
            self.day = Some(day);
        }
    }

    /// Draws `species` (most heard first) on a page of `resolution`. `footer` goes under the
    /// list page; the collage, like fugleramme's, is birds only.
    pub fn draw(
        &mut self,
        species: &[SpeciesSummary],
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        let shown: &[SpeciesSummary] = match self.style.max_birds {
            0 => species,
            limit => &species[..species.len().min(limit)],
        };
        match self.style.layout {
            Layout::List => {
                let heading: String = self
                    .style
                    .heading
                    .clone()
                    .unwrap_or_else(|| self.style.words.heading.clone());
                self.draw_list(species, &heading, footer, resolution)
            }
            Layout::Collage => self.draw_collage(shown, resolution),
        }
    }

    /// Draws the page saying BirdNET-Go cannot be reached, with `problem` underneath.
    pub fn draw_unreachable(&self, problem: &str, resolution: Resolution) -> Canvas {
        let heading: String = self.style.words.unreachable.clone();
        self.draw_list(&[], &heading, problem, resolution)
    }

    fn draw_list(
        &self,
        species: &[SpeciesSummary],
        heading: &str,
        footer: &str,
        resolution: Resolution,
    ) -> Canvas {
        let options = TextListOptions {
            heading: heading.to_owned(),
            footer: footer.to_owned(),
            max_species: match self.style.max_birds {
                0 => 24,
                limit => limit.max(24),
            },
            empty: self.style.words.nothing_heard.clone(),
            more: self.style.words.more.clone(),
        };
        let layout: PageLayout = TextList::layout(species, resolution, &options);
        self.renderer.render(&layout, &[])
    }

    fn draw_collage(&mut self, species: &[SpeciesSummary], resolution: Resolution) -> Canvas {
        for row in species {
            self.ensure_picture(&row.scientific_name.to_string());
        }
        // Species without a picture are left off: with names under the birds, there is nothing
        // to draw for them.
        let pictured: Vec<(&SpeciesSummary, &Picture)> = species
            .iter()
            .filter_map(|row| {
                let picture: &Picture = self
                    .pictures
                    .get(&row.scientific_name.to_string())?
                    .as_ref()?;
                Some((row, picture))
            })
            .collect();
        let masses: Vec<f32> = pictured
            .iter()
            .map(|(row, _)| self.sizes.mass_grams(&row.scientific_name.to_string()))
            .collect();
        let weights: Vec<f32> = size_weights(&masses);
        let mut art: Vec<ArtImage> = Vec::with_capacity(pictured.len());
        let mut birds: Vec<CollageBird> = Vec::with_capacity(pictured.len());
        for ((row, picture), weight) in pictured.into_iter().zip(weights) {
            let name: String = row.scientific_name.to_string();
            let label: String = if row.common_name.is_empty() {
                name.clone()
            } else {
                row.common_name.clone()
            };
            birds.push(CollageBird {
                label_width_em: self.renderer.measure(&label, 1.0),
                label,
                weight,
                span_ratio: picture.art.span_ratio(),
                silhouette: picture.silhouette.clone(),
                mirrored: mirrored(&name),
            });
            art.push(picture.art.clone());
        }
        let options = CollageOptions {
            heading: self.style.heading.clone().unwrap_or_default(),
            empty: self.style.words.nothing_heard.clone(),
            names: self.style.names,
            ..CollageOptions::default()
        };
        let layout: PageLayout = Collage::layout(&birds, resolution, &options);
        self.renderer.render(&layout, &art)
    }

    /// Fetches the species' picture into the cache, if it is not there yet.
    fn ensure_picture(&mut self, scientific_name: &str) {
        if self.pictures.contains_key(scientific_name) {
            return;
        }
        let picture: Option<Picture> = self.art.art_for(scientific_name).and_then(|art| {
            let alpha: Vec<u8> = art.pixels().iter().map(|pixel| pixel.alpha).collect();
            let silhouette: Silhouette = Silhouette::new(art.resolution(), &alpha)?;
            Some(Picture { art, silhouette })
        });
        self.pictures.insert(scientific_name.to_owned(), picture);
    }
}

/// Whether to draw a species mirrored: decided by its name alone, for variety that does not
/// change whenever another bird arrives. FNV-1a, stable across runs.
fn mirrored(scientific_name: &str) -> bool {
    let hash: u64 = scientific_name
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    hash & 0x80 == 0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mirroring_is_stable_and_mixed() {
        let names: [&str; 6] = [
            "Parus major",
            "Erithacus rubecula",
            "Pica pica",
            "Turdus merula",
            "Columba palumbus",
            "Passer domesticus",
        ];
        let flipped: usize = names.iter().filter(|name| mirrored(name)).count();
        assert!(flipped > 0 && flipped < names.len());
        assert_eq!(mirrored("Parus major"), mirrored("Parus major"));
    }
}
