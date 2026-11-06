//! The illustrated page: every bird heard, packed by its outline, heavier birds bigger.
//!
//! Ported from fugleramme's `render/collage.py` (MIT, © 2026 Arne Giacomo Munthe-Kaas).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use frame_core::Resolution;

use crate::align::Align;
use crate::corner_sprays::{CornerSprays, PlacedSpray};
use crate::footprint::{Anchors, Footprint};
use crate::image_block::ImageBlock;
use crate::page_frame::{self, Body, Furniture};
use crate::page_layout::PageLayout;
use crate::silhouette::Silhouette;
use crate::spiral::{self, Blocked};
use crate::text_block::TextBlock;
use crate::text_style::TextStyle;

/// The short side the page is packed at, whatever its real size, so a page packs the same at
/// every resolution.
const PACK_SHORT: f32 = 1200.0;
/// The most of the page's short side one bird may take.
const MAX_SHARE: f32 = 0.7;
/// Times the set is shrunk by [`SHRINK`] looking for a size that fits.
const ATTEMPTS: i32 = 20;
const SHRINK: f32 = 0.9;
/// When the first size fits with room to spare, the set grows by this much a step, up to
/// [`GROW_STEPS`] times, so a page of a few birds fills it instead of leaving bare paper.
const GROW: f32 = 1.06;
const GROW_STEPS: i32 = 8;
/// Halvings tried between the first size that fits and the one before it.
const SIZE_REFINE: usize = 3;
/// The smallest a bird is drawn, in pack pixels.
const SMALLEST: usize = 24;
/// The smallest a name is set, in pack pixels.
const SMALLEST_NAME: f32 = 11.0;
/// A name's reserved height and its baseline within it, in ems.
const NAME_HEIGHT: f32 = 1.15;
const NAME_BASELINE: f32 = 0.85;
/// Name size against the middle bird's longest side.
const NAME_PER_BIRD: f32 = 0.09;
/// Halvings tried while growing names back, after trying the full size.
const NAME_REFINE: usize = 3;
/// Space kept beside a name, split between its two sides, in ems, so neighbouring names
/// never run together.
const NAME_SPACE: f32 = 0.6;
/// Space between a bird and its name, in ems.
const NAME_GAP: f32 = 0.35;
/// Paper kept between a border's plant and the birds, in pack pixels.
const SPRAY_GAP: usize = 10;

/// One bird to place.
#[derive(Clone, Debug, PartialEq)]
pub struct CollageBird {
    /// The name written under the picture.
    pub label: String,
    /// The name's width when set at one pixel, as the renderer measures it.
    pub label_width_em: f32,
    /// Relative size, e.g. from [`crate::size_weights`]; 1.0 is average.
    pub weight: f32,
    /// How much longer the picture's longest side is than the bird's (see
    /// `ArtImage::span_ratio`), so a bird on a big branch is not drawn small. 1.0 if unknown.
    pub span_ratio: f32,
    /// The picture's outline.
    pub silhouette: Silhouette,
    /// Whether to draw the bird mirrored. Decide it from the species alone, so a bird does not
    /// flip whenever another arrives.
    pub mirrored: bool,
}

/// Settings for [`Collage::layout`].
#[derive(Clone, Debug, PartialEq)]
pub struct CollageOptions {
    /// A heading at the top of the page. Empty for none, the default.
    pub heading: String,
    /// A small line at the foot, e.g. when the page was drawn. Empty for none.
    pub footer: String,
    /// The border, if any.
    pub sprays: CornerSprays,
    /// A line about the main bird, over the footer. Empty for none, the default.
    pub fact: String,
    /// The heading's width at a size of one pixel, so it can sit in the middle of the page
    /// clear of the border's top spray. 0, the default, centres it beside the spray.
    pub heading_width_em: f32,
    /// The fact's width at a size of one pixel, so it can sit in the middle of the page clear
    /// of the border's bottom spray. 0, the default, centres it beside the spray.
    pub fact_width_em: f32,
    /// The line shown when nothing has been heard.
    pub empty: String,
    /// Whether to write each bird's name under it.
    pub names: bool,
    /// Bare paper along each edge, as a fraction of the short side.
    pub margin: f32,
    /// Name size, as a fraction of the short side.
    pub name_size: f32,
}

impl Default for CollageOptions {
    fn default() -> Self {
        Self {
            heading: String::new(),
            footer: String::new(),
            fact: String::new(),
            heading_width_em: 0.0,
            fact_width_em: 0.0,
            sprays: CornerSprays::default(),
            empty: String::from("No birds heard yet"),
            names: true,
            margin: 0.04,
            name_size: 0.032,
        }
    }
}

/// Packs birds by their outlines, outwards from the middle of the page, biggest first.
///
/// Each bird is sized by its weight. The whole set starts larger than the page can hold and
/// shrinks until every bird fits, so a few birds fill the page and many birds pack small.
/// Opaque pixels never overlap and nothing runs off the page, but transparent margins do, so
/// birds nestle closely. Each name is tucked under its bird and packed with it, so it never
/// lands on a neighbour. The cluster is then centred. [`ImageBlock::art`] is the bird's index
/// in the input.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collage;

/// A bird ready to pack at one size.
struct Sprite {
    index: usize,
    size: (usize, usize),
    footprint: Footprint,
    anchors: Anchors,
}

impl Collage {
    /// Returns the page for `birds` at `resolution`.
    #[must_use]
    pub fn layout(
        birds: &[CollageBird],
        resolution: Resolution,
        options: &CollageOptions,
    ) -> PageLayout {
        let mut page = PageLayout::new(resolution);
        let body: Body = page_frame::frame(
            &mut page,
            &Furniture {
                heading: &options.heading,
                fact: &options.fact,
                footer: &options.footer,
                sprays: &options.sprays,
                packed_round: true,
                widths_em: (options.heading_width_em, options.fact_width_em),
            },
        );
        if birds.is_empty() {
            page_frame::nothing_heard(&mut page, &body, &options.empty);
            return page;
        }

        let (width, height) = (resolution.width as f32, resolution.height as f32);
        let scale: f32 = width.min(height) / PACK_SHORT;
        let pack: (f32, f32) = (width / scale, height / scale);
        let margin: f32 = width.min(height) * options.margin;
        let top: f32 = if options.heading.is_empty() {
            margin
        } else {
            body.top as f32
        };
        let bottom: f32 = if options.footer.is_empty() && options.fact.is_empty() {
            height - margin
        } else {
            body.bottom as f32
        };
        let window: Window = Window {
            left: margin / scale,
            top: top / scale,
            width: ((width - 2.0 * margin) / scale).max(1.0) as usize,
            height: ((bottom - top) / scale).max(1.0) as usize,
        };

        let weights: Vec<f32> = birds
            .iter()
            .map(|bird| bird.weight.max(0.01) * bird.span_ratio.max(1.0))
            .collect();
        let mut order: Vec<usize> = (0..birds.len()).collect();
        order.sort_by(|&first, &second| weights[second].total_cmp(&weights[first]));
        let squares: f32 = weights.iter().map(|weight| weight * weight).sum();
        let heaviest: f32 = weights.iter().copied().fold(0.0, f32::max);
        let base: f32 = libm::sqrtf(pack.0 * pack.1 * 1.5 / squares)
            .min(pack.0.min(pack.1) * MAX_SHARE / heaviest);
        let name_px: f32 = (pack.0.min(pack.1) * options.name_size).max(SMALLEST_NAME);

        let blocked: Vec<Blocked> = body
            .sprays
            .iter()
            .map(|placed| keep_clear(placed, scale, &window))
            .collect();
        let packer = Packer {
            birds,
            weights: &weights,
            order: &order,
            base,
            window: &window,
            blocked: &blocked,
        };
        let packed: Option<Packed> = options
            .names
            .then(|| packer.fit(Some(name_px)))
            .flatten()
            // Birds beat blank paper: without names they may still fit.
            .or_else(|| packer.fit(None));
        let Some((sprites, corners, used_px)) = packed else {
            return page;
        };

        for (sprite, &(x, y)) in sprites.iter().zip(&corners) {
            let bird: &CollageBird = &birds[sprite.index];
            let left: f32 = window.left + (x + sprite.anchors.art.0) as f32;
            let top: f32 = window.top + (y + sprite.anchors.art.1) as f32;
            page.images.push(ImageBlock {
                art: sprite.index,
                x: libm::roundf(left * scale) as i32,
                y: libm::roundf(top * scale) as i32,
                width: libm::roundf(sprite.size.0 as f32 * scale) as i32,
                height: libm::roundf(sprite.size.1 as f32 * scale) as i32,
                mirrored: bird.mirrored,
            });
        }
        // Names after every bird, so no neighbour's halo is drawn over one.
        for (sprite, &(x, y)) in sprites.iter().zip(&corners) {
            let Some((label_x, label_y, label_width)) = sprite.anchors.label else {
                continue;
            };
            let centre: f32 = window.left + (x + label_x) as f32 + label_width as f32 / 2.0;
            let top: f32 = window.top + (y + label_y) as f32;
            page.text.push(TextBlock {
                text: birds[sprite.index].label.clone(),
                x: libm::roundf(centre * scale) as i32,
                baseline_y: libm::roundf((top + used_px * NAME_BASELINE) * scale) as i32,
                size_px: used_px * scale,
                max_width: Some(libm::ceilf(label_width as f32 * scale) as i32),
                align: Align::Centre,
                style: TextStyle::Name,
            });
        }
        page
    }
}

/// Packed sprites, their corners, and the name size they were packed with.
type Packed = (Vec<Sprite>, Vec<(usize, usize)>, f32);

/// Returns the pixels `placed` covers, in the window's pack pixels, widened by [`SPRAY_GAP`].
fn keep_clear(placed: &PlacedSpray, scale: f32, window: &Window) -> Blocked {
    let block = &placed.block;
    let (source_width, source_height) = (
        usize::try_from(block.width).unwrap_or(0).max(1),
        usize::try_from(block.height).unwrap_or(0).max(1),
    );
    let gap: usize = SPRAY_GAP;
    let width: usize = (block.width as f32 / scale) as usize + 1;
    let height: usize = (block.height as f32 / scale) as usize + 1;
    let (padded_width, padded_height) = (width + 2 * gap, height + 2 * gap);
    let mut cells: Vec<bool> = vec![false; padded_width * padded_height];
    for y in 0..height {
        let source_y: usize = (y * source_height / height).min(source_height - 1);
        for x in 0..width {
            let source_x: usize = (x * source_width / width).min(source_width - 1);
            if !placed.opaque[source_y * source_width + source_x] {
                continue;
            }
            // A square of gap round each covered pixel: cheap enough for a corner's plant.
            for row in y..=y + 2 * gap {
                let start: usize = row * padded_width + x;
                cells[start..=start + 2 * gap].fill(true);
            }
        }
    }
    Blocked {
        x: libm::roundf(block.x as f32 / scale - window.left) as i32 - gap as i32,
        y: libm::roundf(block.y as f32 / scale - window.top) as i32 - gap as i32,
        width: padded_width,
        height: padded_height,
        cells,
    }
}

/// The name size `packed` used, or none when names are off.
fn packed_name(packed: &Packed, name_px: Option<f32>) -> Option<f32> {
    name_px.map(|_| packed.2)
}

/// The area birds are packed into, in pack pixels.
struct Window {
    left: f32,
    top: f32,
    width: usize,
    height: usize,
}

struct Packer<'a> {
    birds: &'a [CollageBird],
    weights: &'a [f32],
    order: &'a [usize],
    base: f32,
    window: &'a Window,
    /// The border's plants, which birds are packed round.
    blocked: &'a [Blocked],
}

impl Packer<'_> {
    /// Shrinks the set until every bird, name included, fits. Names shrink with the set, as in
    /// fugleramme: a fixed-size name never yields, so a full page of them could not converge.
    /// Once the birds fit, names left small beside big birds (a page of only a few) are grown
    /// back towards a share of the middle bird's size, as far as the packing allows. Returns
    /// the sprites, their corners (centred) and the name size used.
    fn fit(&self, name_px: Option<f32>) -> Option<Packed> {
        let (step, mut shrink, mut packed): (i32, f32, Packed) =
            (0..ATTEMPTS).find_map(|step| {
                let shrink: f32 = libm::powf(SHRINK, step as f32);
                let px: Option<f32> = name_px.map(|px| (px * shrink).max(SMALLEST_NAME));
                Some((step, shrink, self.attempt(shrink, px)?))
            })?;
        if step == 0 {
            // Room to spare: grow until the next step would not fit or a bird gets too big.
            let short: f32 = self.window.width.min(self.window.height) as f32;
            let heaviest: f32 = self.weights.iter().copied().fold(0.0, f32::max);
            for _ in 0..GROW_STEPS {
                let bigger: f32 = shrink * GROW;
                if self.base * bigger * heaviest > short * MAX_SHARE {
                    break;
                }
                match self.attempt(bigger, packed_name(&packed, name_px)) {
                    Some(grown) => (shrink, packed) = (bigger, grown),
                    None => break,
                }
            }
        } else {
            // The step before did not fit; the best size lies between. Halve the gap a few
            // times, so the birds come out as big as the page allows, out to its corners.
            let mut fails: f32 = shrink / SHRINK;
            for _ in 0..SIZE_REFINE {
                let middle: f32 = (shrink + fails) / 2.0;
                let px: Option<f32> = name_px.map(|px| (px * middle).max(SMALLEST_NAME));
                match self.attempt(middle, px) {
                    Some(grown) => (shrink, packed) = (middle, grown),
                    None => fails = middle,
                }
            }
        }
        let Some(largest) = name_px else {
            return Some(packed);
        };
        let wanted: f32 = (self.median_size(shrink) * NAME_PER_BIRD).min(largest);
        if wanted <= packed.2 + 1.0 {
            return Some(packed);
        }
        if let Some(grown) = self.attempt(shrink, Some(wanted)) {
            return Some(grown);
        }
        let (mut fits, mut fails, mut best) = (packed.2, wanted, packed);
        for _ in 0..NAME_REFINE {
            if fails - fits < 1.0 {
                break;
            }
            let middle: f32 = (fits + fails) / 2.0;
            match self.attempt(shrink, Some(middle)) {
                Some(grown) => {
                    fits = middle;
                    best = grown;
                }
                None => fails = middle,
            }
        }
        Some(best)
    }

    /// Packs every bird at `shrink`, with names at `name_px` if any.
    fn attempt(&self, shrink: f32, name_px: Option<f32>) -> Option<Packed> {
        let sprites: Vec<Sprite> = self
            .order
            .iter()
            .map(|&index| self.sprite(index, shrink, name_px))
            .collect();
        let footprints: Vec<Footprint> = sprites
            .iter()
            .map(|sprite| sprite.footprint.clone())
            .collect();
        let (width, height) = (self.window.width, self.window.height);
        let corners = spiral::pack(&footprints, width, height, self.blocked)?;
        // Centring the cluster may move a bird onto a border's plant; it then stays put.
        let centred: Vec<(usize, usize)> = self.centred(&footprints, corners.clone());
        let corners: Vec<(usize, usize)> = if self.blocked.is_empty()
            || spiral::clear(&footprints, &centred, width, height, self.blocked)
        {
            centred
        } else {
            corners
        };
        Some((sprites, corners, name_px.unwrap_or(0.0)))
    }

    /// The longest side bird `index` is drawn with at `shrink`, in pack pixels.
    fn longest_side(&self, index: usize, shrink: f32) -> usize {
        ((self.base * shrink * self.weights[index]) as usize).max(SMALLEST)
    }

    /// The middle bird's longest side at `shrink`.
    fn median_size(&self, shrink: f32) -> f32 {
        let mut sizes: Vec<usize> = (0..self.birds.len())
            .map(|index| self.longest_side(index, shrink))
            .collect();
        sizes.sort_unstable();
        sizes.get(sizes.len() / 2).copied().unwrap_or(SMALLEST) as f32
    }

    fn sprite(&self, index: usize, shrink: f32, name_px: Option<f32>) -> Sprite {
        let bird: &CollageBird = &self.birds[index];
        let longest: usize = self.longest_side(index, shrink);
        let (width, height) = bird.silhouette.scaled_size(longest);
        let art = Footprint::of_art(
            width,
            height,
            bird.silhouette.mask(width, height, bird.mirrored),
        );
        let (footprint, anchors) = match name_px {
            Some(px) => {
                let label_width: usize =
                    libm::ceilf((bird.label_width_em + NAME_SPACE) * px) as usize;
                let label_height: usize = libm::ceilf(px * NAME_HEIGHT) as usize;
                art.with_label(
                    label_width,
                    label_height,
                    libm::roundf(px * NAME_GAP) as usize,
                )
            }
            None => (
                art,
                Anchors {
                    art: (0, 0),
                    label: None,
                },
            ),
        };
        Sprite {
            index,
            size: (width, height),
            footprint,
            anchors,
        }
    }

    /// Moves the packed cluster so its bounding box is centred in the window.
    fn centred(
        &self,
        footprints: &[Footprint],
        corners: Vec<(usize, usize)>,
    ) -> Vec<(usize, usize)> {
        let left: usize = corners.iter().map(|corner| corner.0).min().unwrap_or(0);
        let top: usize = corners.iter().map(|corner| corner.1).min().unwrap_or(0);
        let right: usize = footprints
            .iter()
            .zip(&corners)
            .map(|(footprint, corner)| corner.0 + footprint.width)
            .max()
            .unwrap_or(0);
        let bottom: usize = footprints
            .iter()
            .zip(&corners)
            .map(|(footprint, corner)| corner.1 + footprint.height)
            .max()
            .unwrap_or(0);
        let new_left: usize = (self.window.width - (right - left)) / 2;
        let new_top: usize = (self.window.height - (bottom - top)) / 2;
        corners
            .into_iter()
            .map(|(x, y)| (x - left + new_left, y - top + new_top))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::corner_sprays::Spray;

    /// A bird-ish outline: an ellipse filling most of a `width` by `height` picture.
    fn outline(width: usize, height: usize) -> Silhouette {
        let alpha: Vec<u8> = (0..width * height)
            .map(|index| {
                let x: f32 = (index % width) as f32 / width as f32 - 0.5;
                let y: f32 = (index / width) as f32 / height as f32 - 0.5;
                if x * x + y * y < 0.2 { 255 } else { 0 }
            })
            .collect();
        Silhouette::new(Resolution::new(width, height), &alpha).expect("sizes match")
    }

    fn bird(label: &str, weight: f32) -> CollageBird {
        CollageBird {
            label: String::from(label),
            label_width_em: label.len() as f32 * 0.42,
            weight,
            span_ratio: 1.0,
            silhouette: outline(120, 90),
            mirrored: false,
        }
    }

    fn garden(count: usize) -> Vec<CollageBird> {
        (0..count)
            .map(|index| bird("Great Spotted Woodpecker", 0.7 + (index % 5) as f32 * 0.15))
            .collect()
    }

    #[test]
    fn empty_page_says_so() {
        let options = CollageOptions {
            empty: String::from("Inga fåglar än"),
            ..CollageOptions::default()
        };
        let page = Collage::layout(&[], Resolution::new(1600, 1200), &options);
        assert!(page.images.is_empty());
        assert!(page.text.iter().any(|block| block.text == "Inga fåglar än"));
    }

    #[test]
    fn birds_and_names_stay_inside_the_page() {
        for resolution in [Resolution::new(1600, 1200), Resolution::new(1200, 1600)] {
            for count in [1, 2, 9, 30] {
                let page = Collage::layout(&garden(count), resolution, &CollageOptions::default());
                assert_eq!(page.images.len(), count);
                assert_eq!(page.text.len(), count, "a name per bird, no heading");
                let (width, height) = (resolution.width as i32, resolution.height as i32);
                for image in &page.images {
                    assert!(image.x >= 0 && image.y >= 0, "{image:?}");
                    assert!(image.x + image.width <= width && image.y + image.height <= height);
                }
                for name in &page.text {
                    assert!(name.baseline_y < height && name.x > 0 && name.x < width);
                }
                let mut arts: Vec<usize> = page.images.iter().map(|image| image.art).collect();
                arts.sort_unstable();
                assert_eq!(arts, (0..count).collect::<Vec<usize>>());
            }
        }
    }

    #[test]
    fn biggest_bird_is_in_the_middle_and_heavier_birds_are_bigger() {
        let page = Collage::layout(
            &[bird("Wren", 0.7), bird("Pigeon", 1.4), bird("Robin", 1.0)],
            Resolution::new(1600, 1200),
            &CollageOptions::default(),
        );
        let image = |art: usize| -> ImageBlock {
            *page
                .images
                .iter()
                .find(|image| image.art == art)
                .expect("placed")
        };
        assert!(image(1).width > image(2).width && image(2).width > image(0).width);
        let pigeon = image(1);
        let middle: i32 = pigeon.x + pigeon.width / 2;
        assert!((600..1000).contains(&middle), "{pigeon:?}");
    }

    #[test]
    fn a_heading_and_footer_take_their_own_space() {
        let options = CollageOptions {
            heading: String::from("Heard in the garden"),
            footer: String::from("Updated 07:00"),
            ..CollageOptions::default()
        };
        let page = Collage::layout(&garden(3), Resolution::new(1600, 1200), &options);
        let heading: &TextBlock = &page.text[0];
        assert_eq!(heading.style, TextStyle::Heading);
        for image in &page.images {
            assert!(image.y > heading.baseline_y, "{image:?}");
        }
    }

    #[test]
    fn names_can_be_left_off() {
        let options = CollageOptions {
            names: false,
            ..CollageOptions::default()
        };
        let page = Collage::layout(&garden(4), Resolution::new(1600, 1200), &options);
        assert_eq!(page.images.len(), 4);
        assert!(page.text.is_empty());
    }

    /// Sprays filling a square: a solid triangle from the corner they sit in.
    fn sprays(first_art: usize) -> CornerSprays {
        let side: usize = 100;
        let corner = |top_left: bool| -> Spray {
            let alpha: Vec<u8> = (0..side * side)
                .map(|index| {
                    let (x, y) = (index % side, index / side);
                    let solid: bool = if top_left {
                        x + y < side
                    } else {
                        x + y >= side
                    };
                    if solid { 255 } else { 0 }
                })
                .collect();
            Spray {
                art: first_art + usize::from(!top_left),
                silhouette: Silhouette::new(Resolution::new(side, side), &alpha).expect("sizes"),
            }
        };
        CornerSprays {
            top_left: Some(corner(true)),
            bottom_right: Some(corner(false)),
        }
    }

    #[test]
    fn birds_are_packed_round_the_borders_plants() {
        for resolution in [Resolution::new(1600, 1200), Resolution::new(800, 480)] {
            for count in [3, 30] {
                let options = CollageOptions {
                    fact: String::from("Great Tit: found as far north as 71° N"),
                    sprays: sprays(count),
                    ..CollageOptions::default()
                };
                let page = Collage::layout(&garden(count), resolution, &options);
                assert_eq!(page.images.len(), count + 2, "every bird and both sprays");
                let plants: Vec<ImageBlock> = page
                    .images
                    .iter()
                    .filter(|image| image.art >= count)
                    .copied()
                    .collect();
                assert_eq!(plants.len(), 2);
                let (top, bottom) = (plants[0], plants[1]);
                assert!(top.x < 30 && top.y < 30, "{top:?} hangs from the top-left");
                let (width, height) = (resolution.width as i32, resolution.height as i32);
                assert!(width - (bottom.x + bottom.width) < 30, "{bottom:?}");
                assert!(height - (bottom.y + bottom.height) < 30, "{bottom:?}");
                // The plants are triangles in their corners: a bird's box may reach into the
                // square, but not past the diagonal into the plant.
                for bird in page.images.iter().filter(|image| image.art < count) {
                    let (centre_x, centre_y) = (bird.x + bird.width / 2, bird.y + bird.height / 2);
                    let in_top: bool = (centre_x - top.x) + (centre_y - top.y) < top.width;
                    let in_bottom: bool = (bottom.x + bottom.width - centre_x)
                        + (bottom.y + bottom.height - centre_y)
                        < bottom.width;
                    assert!(!in_top && !in_bottom, "{bird:?} sits on a plant");
                }
                let fact = page
                    .text
                    .iter()
                    .find(|block| block.text.starts_with("Great Tit:"))
                    .expect("the fact");
                let room: i32 = fact.max_width.expect("a width");
                assert!(
                    fact.x + room / 2 <= bottom.x,
                    "the fact line clears the plant"
                );
            }
        }
    }
}
