//! Floyd-Steinberg dithering to the six inks.

use alloc::vec;
use alloc::vec::Vec;

use frame_core::{IndexedImage, Ink, Resolution, Rgb};

use crate::canvas::Canvas;

/// Reduces `canvas` to the six inks with Floyd-Steinberg error diffusion, measuring colour
/// distance against [`Ink::dither_colour`].
#[must_use]
#[expect(
    clippy::expect_used,
    clippy::missing_panics_doc,
    reason = "cannot fail: one ink per pixel"
)]
pub fn dither(canvas: &Canvas) -> IndexedImage {
    let Resolution { width, height } = canvas.resolution();
    let mut current: Vec<[i32; 3]> = vec![[0; 3]; width + 2];
    let mut next: Vec<[i32; 3]> = vec![[0; 3]; width + 2];
    let mut inks: Vec<Ink> = Vec::with_capacity(width * height);
    let source: &[Rgb] = canvas.pixels();

    for y in 0..height {
        for x in 0..width {
            let pixel: Rgb = source[y * width + x];
            let carried: [i32; 3] = current[x + 1];
            let wanted: [i32; 3] = [
                (i32::from(pixel.red) + carried[0]).clamp(0, 255),
                (i32::from(pixel.green) + carried[1]).clamp(0, 255),
                (i32::from(pixel.blue) + carried[2]).clamp(0, 255),
            ];
            let ink: Ink = nearest(wanted);
            inks.push(ink);
            let shown: Rgb = ink.dither_colour();
            let error: [i32; 3] = [
                wanted[0] - i32::from(shown.red),
                wanted[1] - i32::from(shown.green),
                wanted[2] - i32::from(shown.blue),
            ];
            for channel in 0..3 {
                current[x + 2][channel] += error[channel] * 7 / 16;
                next[x][channel] += error[channel] * 3 / 16;
                next[x + 1][channel] += error[channel] * 5 / 16;
                next[x + 2][channel] += error[channel] / 16;
            }
        }
        core::mem::swap(&mut current, &mut next);
        next.fill([0; 3]);
    }
    IndexedImage::from_pixels(canvas.resolution(), inks)
        .expect("one ink was pushed per canvas pixel")
}

fn nearest(wanted: [i32; 3]) -> Ink {
    let mut best: Ink = Ink::White;
    let mut best_distance: i32 = i32::MAX;
    for ink in Ink::ALL {
        let colour: Rgb = ink.dither_colour();
        let red: i32 = wanted[0] - i32::from(colour.red);
        let green: i32 = wanted[1] - i32::from(colour.green);
        let blue: i32 = wanted[2] - i32::from(colour.blue);
        let distance: i32 = red * red + green * green + blue * blue;
        if distance < best_distance {
            best = ink;
            best_distance = distance;
        }
    }
    best
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn white_paper_stays_white() {
        let canvas = Canvas::new(Resolution::new(64, 8), Rgb::WHITE);
        assert!(
            dither(&canvas)
                .pixels()
                .iter()
                .all(|&ink| ink == Ink::White)
        );
    }

    #[test]
    fn exact_inks_are_kept() {
        let mut canvas = Canvas::new(Resolution::new(6, 1), Rgb::WHITE);
        for (x, ink) in Ink::ALL.iter().enumerate() {
            canvas.blend(
                i32::try_from(x).expect("small"),
                0,
                ink.dither_colour(),
                255,
            );
        }
        assert_eq!(dither(&canvas).pixels(), &Ink::ALL);
    }
}
