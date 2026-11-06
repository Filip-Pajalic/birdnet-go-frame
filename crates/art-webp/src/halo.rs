//! Making every plate look printed on the same paper.
//!
//! Adapted from fugleramme's `render/paper.py` (MIT, © 2026 Arne Giacomo Munthe-Kaas). The old
//! plates keep a ring of their scanned paper around each bird, in a tone that varies from scan
//! to scan. The ring's tone is measured, paper-like pixels are shifted to the page's paper, and
//! the cut edge is softened inwards so the ring melts into the page.

use art_pack::{ArtImage, Rgba};
use frame_core::{Resolution, Rgb};

/// Alpha above which a pixel counts as part of the plate.
const OPAQUE: u8 = 24;
/// How far from the cut the paper ring is sampled, in pixels.
const RING: usize = 4;
/// Box-blur radius of the soft edge; three passes approximate a Gaussian.
const FEATHER: usize = 4;

/// Returns `picture` with its paper halo toned to `paper` and its edge feathered.
pub(crate) fn blend_into(picture: &ArtImage, paper: Rgb) -> ArtImage {
    let Resolution { width, height } = picture.resolution();
    let mut pixels: Vec<Rgba> = picture.pixels().to_vec();
    let opaque: Vec<bool> = pixels.iter().map(|pixel| pixel.alpha > OPAQUE).collect();
    let near_cut: Vec<bool> = near_transparent(&opaque, width, height);
    let ring: Vec<usize> = (0..pixels.len())
        .filter(|&index| opaque[index] && near_cut[index])
        .collect();

    let scan_paper: [i32; 3] = if ring.len() > 50 {
        let median = |channel: fn(&Rgba) -> u8| -> i32 {
            let mut values: Vec<u8> = ring.iter().map(|&index| channel(&pixels[index])).collect();
            values.sort_unstable();
            i32::from(values[values.len() / 2])
        };
        [
            median(|pixel| pixel.red),
            median(|pixel| pixel.green),
            median(|pixel| pixel.blue),
        ]
    } else {
        [
            i32::from(paper.red),
            i32::from(paper.green),
            i32::from(paper.blue),
        ]
    };
    let target: [i32; 3] = [
        i32::from(paper.red),
        i32::from(paper.green),
        i32::from(paper.blue),
    ];

    for (index, pixel) in pixels.iter_mut().enumerate() {
        if !opaque[index] {
            // The feathered edge reveals this colour, so make it paper.
            *pixel = Rgba::new(paper.red, paper.green, paper.blue, pixel.alpha);
            continue;
        }
        let rgb: [i32; 3] = [
            i32::from(pixel.red),
            i32::from(pixel.green),
            i32::from(pixel.blue),
        ];
        let distance: i32 = (0..3)
            .map(|channel| (rgb[channel] - scan_paper[channel]).abs())
            .max()
            .unwrap_or(0);
        let brightest: i32 = rgb.iter().copied().max().unwrap_or(0);
        let saturation: i32 = brightest - rgb.iter().copied().min().unwrap_or(0);
        let paper_like: bool = distance < 50 && saturation < 55 && brightest > 160;
        if !paper_like {
            continue;
        }
        let shifted = |channel: usize| -> u8 {
            if near_cut[index] {
                // The outer ring keeps a bright fringe from cutting and scaling; snap it flat.
                u8::try_from(target[channel]).unwrap_or(u8::MAX)
            } else {
                u8::try_from((rgb[channel] + target[channel] - scan_paper[channel]).clamp(0, 255))
                    .unwrap_or(u8::MAX)
            }
        };
        *pixel = Rgba::new(shifted(0), shifted(1), shifted(2), pixel.alpha);
    }

    // Feather inwards only, so the plate's footprint does not grow.
    let mut alpha: Vec<u16> = pixels.iter().map(|pixel| u16::from(pixel.alpha)).collect();
    for _ in 0..3 {
        alpha = box_blur(&alpha, width, height, FEATHER);
    }
    for (pixel, soft) in pixels.iter_mut().zip(alpha) {
        pixel.alpha = pixel.alpha.min(u8::try_from(soft).unwrap_or(u8::MAX));
    }
    ArtImage::new(picture.resolution(), pixels)
        .map(|toned| match picture.subject() {
            Some(subject) => toned.with_subject(subject),
            None => toned,
        })
        .unwrap_or_else(|| picture.clone())
}

/// Marks pixels within [`RING`] of a transparent pixel or of the picture's edge.
fn near_transparent(opaque: &[bool], width: usize, height: usize) -> Vec<bool> {
    let clear: Vec<u16> = opaque
        .iter()
        .map(|&solid| if solid { 0 } else { 255 })
        .collect();
    let spread: Vec<u16> = box_blur(&clear, width, height, RING);
    let near_edge = |x: usize, y: usize| -> bool {
        x < RING || y < RING || x + RING >= width || y + RING >= height
    };
    spread
        .iter()
        .enumerate()
        .map(|(index, &value)| value > 0 || near_edge(index % width, index / width))
        .collect()
}

/// A separable box blur; outside the picture counts as zero.
fn box_blur(values: &[u16], width: usize, height: usize, radius: usize) -> Vec<u16> {
    let span: u32 = u32::try_from(2 * radius + 1).unwrap_or(1);
    let mut across: Vec<u16> = vec![0; values.len()];
    for y in 0..height {
        let row: &[u16] = &values[y * width..(y + 1) * width];
        let mut sum: u32 = row.iter().take(radius).map(|&value| u32::from(value)).sum();
        for x in 0..width {
            if let Some(&entering) = row.get(x + radius) {
                sum += u32::from(entering);
            }
            across[y * width + x] = u16::try_from(sum.div_ceil(span)).unwrap_or(u16::MAX);
            if x >= radius {
                sum -= u32::from(row[x - radius]);
            }
        }
    }
    let mut down: Vec<u16> = vec![0; values.len()];
    for x in 0..width {
        let mut sum: u32 = (0..radius.min(height))
            .map(|y| u32::from(across[y * width + x]))
            .sum();
        for y in 0..height {
            if y + radius < height {
                sum += u32::from(across[(y + radius) * width + x]);
            }
            down[y * width + x] = u16::try_from(sum.div_ceil(span)).unwrap_or(u16::MAX);
            if y >= radius {
                sum -= u32::from(across[(y - radius) * width + x]);
            }
        }
    }
    down
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tones_the_halo_and_softens_the_edge() {
        // A 60x60 plate: grey-cream scan paper with a dark bird in the middle.
        let size: usize = 60;
        let mut pixels: Vec<Rgba> = vec![Rgba::new(220, 214, 196, 255); size * size];
        for y in 25..35 {
            for x in 25..35 {
                pixels[y * size + x] = Rgba::new(40, 30, 20, 255);
            }
        }
        let plate = ArtImage::new(Resolution::new(size, size), pixels).expect("sizes match");
        let paper = Rgb::new(233, 225, 202);
        let toned = blend_into(&plate, paper);
        let middle_of_halo = toned.get(15, 30);
        assert_eq!(middle_of_halo.rgb(), paper);
        assert_eq!(
            toned.get(30, 30),
            Rgba::new(40, 30, 20, 255),
            "the bird is untouched"
        );
        assert!(
            toned.get(0, 30).alpha < 200,
            "the edge fades: {:?}",
            toned.get(0, 30)
        );
        assert_eq!(middle_of_halo.alpha, 255);
    }
}
