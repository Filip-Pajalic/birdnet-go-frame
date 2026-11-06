//! Darkening the birds before dithering, so pale plumage takes ink.
//!
//! Adapted from fugleramme's `render/dither.py` (MIT, © 2026 Arne Giacomo Munthe-Kaas).

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
const HALO_REACH: u8 = 32;

/// Returns `canvas` with its pictures darkened and given more contrast for the six-ink panel.
///
/// Pale plumage would otherwise dither to bare paper. The paper itself, and the soft paper
/// halos around each plate, keep their tone: darkened, they would ring every bird.
#[must_use]
pub fn pop(canvas: &Canvas, paper: Rgb) -> Canvas {
    let mut popped: Canvas = canvas.clone();
    pop_in_place(&mut popped, paper);
    popped
}

/// Does what [`pop`] does, in place, with one byte of working memory per pixel. For devices
/// such as the ESP32 that cannot hold a second page.
pub fn pop_in_place(canvas: &mut Canvas, paper: Rgb) {
    let Resolution { width, height } = canvas.resolution();
    let paper_rgb: [f32; 3] = channels(paper);
    let steps: Vec<u8> = reach(canvas.pixels(), paper_rgb, width, height);
    for (pixel, step) in canvas.pixels_mut().iter_mut().zip(steps) {
        let keep: f32 = if step <= HALO_REACH {
            1.0
        } else {
            libm::expf(-distance(*pixel, paper_rgb) / FADE)
        };
        let rgb: [f32; 3] = channels(*pixel);
        let out = |channel: usize| -> u8 {
            let value: f32 = boost(rgb[channel], channel)
                + keep * (paper_rgb[channel] - boost(paper_rgb[channel], channel));
            libm::roundf(value.clamp(0.0, 255.0)) as u8
        };
        *pixel = Rgb::new(out(0), out(1), out(2));
    }
}

/// How far `colour` is from the paper's channels.
fn distance(colour: Rgb, paper_rgb: [f32; 3]) -> f32 {
    let rgb: [f32; 3] = channels(colour);
    let squares: f32 = (0..3)
        .map(|channel| (rgb[channel] - paper_rgb[channel]) * (rgb[channel] - paper_rgb[channel]))
        .sum();
    libm::sqrtf(squares)
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

/// Marks a pixel near the paper's colour that no step has reached yet.
const NEAR: u8 = u8::MAX - 1;
/// Marks a pixel too far from the paper's colour to be part of a halo.
const FAR: u8 = u8::MAX;

/// Returns, for each pixel near the paper's colour that can be reached from the paper within
/// [`HALO_REACH`] steps, moving only through such pixels, four-connected so thin ink stops it,
/// how many steps it takes; [`NEAR`] or [`FAR`] for the rest.
///
/// A breadth-first search one ring at a time: each pass over the page reaches the pixels one
/// step further, so no queue is needed.
fn reach(pixels: &[Rgb], paper_rgb: [f32; 3], width: usize, height: usize) -> Vec<u8> {
    let mut steps: Vec<u8> = pixels
        .iter()
        .map(|&pixel| match distance(pixel, paper_rgb) {
            at if at <= 1.0 => 0,
            at if at <= HALO => NEAR,
            _ => FAR,
        })
        .collect();
    for step in 1..=HALO_REACH {
        let mut grew: bool = false;
        for index in 0..steps.len() {
            if steps[index] != NEAR {
                continue;
            }
            let (x, y) = (index % width, index / width);
            let touches: bool = (x > 0 && steps[index - 1] == step - 1)
                || (x + 1 < width && steps[index + 1] == step - 1)
                || (y > 0 && steps[index - width] == step - 1)
                || (y + 1 < height && steps[index + width] == step - 1);
            if touches {
                steps[index] = step;
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    steps
}

#[cfg(test)]
mod test {
    use alloc::collections::VecDeque;
    use alloc::vec;

    use super::*;

    /// The search as first written, with a queue, to check the ring-at-a-time one against.
    fn reach_with_queue(distances: &[f32], width: usize, height: usize) -> Vec<bool> {
        let mut steps: Vec<Option<u8>> = vec![None; distances.len()];
        let mut queue: VecDeque<usize> = VecDeque::new();
        for (index, &distance) in distances.iter().enumerate() {
            if distance <= 1.0 {
                steps[index] = Some(0);
                queue.push_back(index);
            }
        }
        while let Some(index) = queue.pop_front() {
            let step: u8 = steps[index].unwrap_or(0);
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

    #[test]
    fn rings_reach_the_same_pixels_as_a_queue() {
        let paper = Rgb::new(233, 225, 202);
        let paper_rgb: [f32; 3] = channels(paper);
        let (width, height): (usize, usize) = (97, 61);
        let mut seed: u32 = 12_345;
        let pixels: Vec<Rgb> = (0..width * height)
            .map(|index| {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                let off: u8 = u8::try_from((seed >> 16) % 12).unwrap_or(0);
                // Long near-paper corridors, walls of ink every so often.
                if index.is_multiple_of(41) || (seed >> 8).is_multiple_of(23) {
                    Rgb::new(40, 60, 90)
                } else if (seed >> 4).is_multiple_of(3) {
                    paper
                } else {
                    Rgb::new(233 - off, 225 - off / 2, 202)
                }
            })
            .collect();
        let distances: Vec<f32> = pixels
            .iter()
            .map(|&pixel| distance(pixel, paper_rgb))
            .collect();
        let expected: Vec<bool> = reach_with_queue(&distances, width, height);
        let steps: Vec<u8> = reach(&pixels, paper_rgb, width, height);
        let found: Vec<bool> = steps.iter().map(|&step| step <= HALO_REACH).collect();
        assert_eq!(found, expected);
        assert!(found.iter().filter(|&&reached| reached).count() > width * height / 3);
    }

    #[test]
    fn the_queue_search_reaches_only_so_far() {
        // A corridor of near-paper pixels 40 long from one paper pixel: only 32 are reached.
        let mut distances: Vec<f32> = vec![5.0; 41];
        distances[0] = 0.0;
        let reached: usize = reach_with_queue(&distances, 41, 1)
            .iter()
            .filter(|&&reached| reached)
            .count();
        assert_eq!(reached, 33);
        let paper = Rgb::new(233, 225, 202);
        let mut pixels: Vec<Rgb> = vec![Rgb::new(229, 222, 202); 41];
        pixels[0] = paper;
        let steps: Vec<u8> = reach(&pixels, channels(paper), 41, 1);
        assert_eq!(steps.iter().filter(|&&step| step <= HALO_REACH).count(), 33);
    }

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
