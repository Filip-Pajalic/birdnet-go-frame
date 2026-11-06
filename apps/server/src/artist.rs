//! Draws pages: picks the layout, finds each bird's picture and size, renders.

use std::collections::HashMap;

use art_pack::{ArtImage, ArtSource};
use bird_sizes::BirdSizes;
use birdnet_model::SpeciesSummary;
use frame_core::Resolution;
use frame_layout::{
    Collage, CollageBird, CollageOptions, PageLayout, TextList, TextListOptions, size_weights,
};
use frame_raster::{Canvas, Renderer};

use crate::config::Layout;

/// What goes on a page besides the birds.
pub struct Captions<'a> {
    pub heading: &'a str,
    pub footer: &'a str,
}

/// Draws pages with one font, one source of pictures and one layout.
pub struct Artist<A: ArtSource> {
    renderer: Renderer,
    art: A,
    pictures: HashMap<String, ArtImage>,
    sizes: BirdSizes<'static>,
    layout: Layout,
    max_birds: usize,
}

impl<A: ArtSource> Artist<A> {
    pub fn new(renderer: Renderer, art: A, layout: Layout, max_birds: usize) -> Self {
        Self {
            renderer,
            art,
            pictures: HashMap::new(),
            sizes: BirdSizes::bundled(),
            layout,
            max_birds,
        }
    }

    /// Draws `species` (most heard first) on a page of `resolution`.
    pub fn draw(
        &mut self,
        species: &[SpeciesSummary],
        captions: &Captions<'_>,
        resolution: Resolution,
    ) -> Canvas {
        match self.layout {
            Layout::List => self.draw_list(species, captions, resolution),
            Layout::Collage => self.draw_collage(species, captions, resolution),
        }
    }

    /// Draws a page of text only, e.g. to say BirdNET-Go cannot be reached.
    pub fn draw_list(
        &self,
        species: &[SpeciesSummary],
        captions: &Captions<'_>,
        resolution: Resolution,
    ) -> Canvas {
        let options = TextListOptions {
            heading: captions.heading.to_owned(),
            footer: captions.footer.to_owned(),
            max_species: self.max_birds.max(24),
        };
        let layout: PageLayout = TextList::layout(species, resolution, &options);
        self.renderer.render(&layout, &[])
    }

    fn draw_collage(
        &mut self,
        species: &[SpeciesSummary],
        captions: &Captions<'_>,
        resolution: Resolution,
    ) -> Canvas {
        let shown: usize = species.len().min(self.max_birds.max(1));
        let art: Vec<ArtImage> = species[..shown]
            .iter()
            .map(|row| self.picture(&row.scientific_name.to_string()))
            .collect();
        let masses: Vec<f32> = species[..shown]
            .iter()
            .map(|row| self.sizes.mass_grams(&row.scientific_name.to_string()))
            .collect();
        let weights: Vec<f32> = size_weights(&masses);
        let birds: Vec<CollageBird> = species
            .iter()
            .enumerate()
            .map(|(index, row)| CollageBird {
                label: if row.common_name.is_empty() {
                    row.scientific_name.to_string()
                } else {
                    row.common_name.clone()
                },
                weight: weights.get(index).copied().unwrap_or(1.0),
                aspect_ratio: art.get(index).map_or(1.0, ArtImage::aspect_ratio),
            })
            .collect();
        let options = CollageOptions {
            heading: captions.heading.to_owned(),
            footer: captions.footer.to_owned(),
            max_birds: shown,
        };
        let layout: PageLayout = Collage::layout(&birds, resolution, &options);
        self.renderer.render(&layout, &art)
    }

    /// Returns the species' picture, from the cache or the art source. Species without one
    /// get a blank picture, which draws nothing but keeps their name on the page.
    fn picture(&mut self, scientific_name: &str) -> ArtImage {
        if let Some(picture) = self.pictures.get(scientific_name) {
            return picture.clone();
        }
        let picture: ArtImage = self.art.art_for(scientific_name).unwrap_or_else(blank);
        self.pictures
            .insert(scientific_name.to_owned(), picture.clone());
        picture
    }
}

#[expect(
    clippy::expect_used,
    reason = "cannot fail: one pixel for a 1x1 picture"
)]
fn blank() -> ArtImage {
    ArtImage::new(Resolution::new(1, 1), vec![art_pack::Rgba::TRANSPARENT])
        .expect("one pixel for a 1x1 picture")
}
