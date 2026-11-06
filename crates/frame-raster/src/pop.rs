//! Darkening the birds before dithering, so pale plumage takes ink.
//!
//! Adapted from fugleramme's `render/dither.py` (MIT, © 2026 Arne Giacomo Munthe-Kaas).

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use frame_core::{Resolution, Rgb};

use crate::canvas::Canvas;

/// Per-channel darkening, a little more in red and green: the panel's white is bluish grey.
const LIFT: [f32; 3] = [0.887, 0.906, 0.950];
/// The level contrast is stretched about.
const PIVOT: f32 = 182.0;
const CONTRAST: f32 = 1.15;
/// How fast colours near the paper stop being kept as they are, in colour distance.
const FADE: f32 = 4.0;
/// Colours this near the paper count as part of a halo when they touch the paper.
const HALO: f32 = 12.0;
/// How far, in pixels, a halo reaches in from the paper.
const HALO_REACH: u16 = 32;

/// Returns `canvas` with its pictures darkened and given more contrast for the six-ink panel.
///
/// Pale plumage would otherwise dither to bare paper. The paper itself, and the soft paper
/// halos around each plate, keep their tone: darkened, they would ring every bird.
#[must_use]
pub fn pop(canvas: &Canvas, paper: Rgb) -> Canvas {
    let Resolution { width, height } = canvas.resolution();
    let paper_rgb: [f32; 3] = channels(paper);
    let distances: Vec<f32> = canvas
        .pixels()
        .iter()
        .map(|&pixel| {
            let rgb: [f32; 3] = channels(pixel);
            let squares: f32 = (0..3)
                .map(|channel| {
                    (rgb[channel] - paper_rgb[channel]) * (rgb[channel] - paper_rgb[channel])
                })
                .sum();
            libm::sqrtf(squares)
        })
        .collect();
    let halo: Vec<bool> = reach(&distances, width, height);
    let mut pixels: Vec<Rgb> = Vec::with_capacity(distances.len());
    for (index, &pixel) in canvas.pixels().iter().enumerate() {
        let keep: f32 = if halo[index] {
            1.0
        } else {
            libm::expf(-distances[index] / FADE)
        };
        let rgb: [f32; 3] = channels(pixel);
        let out = |channel: usize| -> u8 {
            let value: f32 = boost(rgb[channel], channel)
                + keep * (paper_rgb[channel] - boost(paper_rgb[channel], channel));
            libm::roundf(value.clamp(0.0, 255.0)) as u8
        };
        pixels.push(Rgb::new(out(0), out(1), out(2)));
    }
    Canvas::from_pixels(canvas.resolution(), pixels)
}

fn channels(colour: Rgb) -> [f32; 3] {
    [
        f32::from(colour.red),
        f32::from(colour.green),
        f32::from(colour.blue),
    ]
}

fn boost(value: f32, channel: usize) -> f32 {
    PIVOT + (value * LIFT[channel] - PIVOT) * CONTRAST
}

/// Marks pixels near the paper's colour that can be reached from the paper within
/// [`HALO_REACH`] steps, moving only through such pixels, four-connected so thin ink stops it.
fn reach(distances: &[f32], width: usize, height: usize) -> Vec<bool> {
    let mut steps: Vec<Option<u16>> = vec![None; distances.len()];
    let mut queue: VecDeque<usize> = VecDeque::new();
    for (index, &distance) in distances.iter().enumerate() {
        if distance <= 1.0 {
            steps[index] = Some(0);
            queue.push_back(index);
        }
    }
    while let Some(index) = queue.pop_front() {
        let step: u16 = steps[index].unwrap_or(0);
        if step >= HALO_REACH {
            continue;
        }
        let (x, y) = (index % width, index / width);
        let neighbours: [Option<usize>; 4] = [
            (x > 0).then(|| index - 1),
            (x + 1 < width).then(|| index + 1),
            (y > 0).then(|| index - width),
            (y + 1 < height).then(|| index + width),
        ];
        for next in neighbours.into_iter().flatten() {
            if steps[next].is_none() && distances[next] <= HALO {
                steps[next] = Some(step + 1);
                queue.push_back(next);
            }
        }
    }
    steps.iter().map(Option::is_some).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn paper_and_halo_keep_their_tone_and_birds_darken() {
        let paper = Rgb::new(233, 225, 202);
        let mut canvas = Canvas::new(Resolution::new(30, 1), paper);
        // A halo pixel a little off the paper, then a pale bird.
        canvas.blend(1, 0, Rgb::new(226, 220, 200), u8::MAX);
        for x in 10..20 {
            canvas.blend(x, 0, Rgb::new(200, 190, 170), u8::MAX);
        }
        let popped = pop(&canvas, paper);
        assert_eq!(popped.pixels()[0], paper);
        let halo: Rgb = popped.pixels()[1];
        assert!(
            halo.red >= 224 && halo.green >= 218,
            "barely changed: {halo:?}"
        );
        let bird: Rgb = popped.pixels()[15];
        assert!(bird.red < 190 && bird.green < 180, "{bird:?}");
    }
}
