//! The seasonal border: an engraved ornament round the page's edge in the panel's inks.

use alloc::vec::Vec;
use core::f32::consts::{FRAC_PI_2, PI};

use frame_core::{Ink, Resolution};

use crate::border_theme::BorderTheme;
use crate::engraver::{self, Engraver};
use crate::pen_mark::PenMark;
use crate::point::Point;

/// The outer rule's distance from the edge, as a fraction of the short side.
const OUTER: f32 = 0.014;
/// The inner rule's distance from the edge.
const INNER: f32 = 0.022;

/// Draws a seasonal border round a page, in the style of the plates: a double rule in
/// hairlines and, along it, motifs outlined in black and hatched in colour. Everything stays
/// within the outer few percent of the page, inside the margin pages keep clear, apart from
/// the corners' motifs, and is drawn under the pictures and the words.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeasonBorder;

/// A page's size and the corners and edges motifs are placed on.
struct Frame {
    width: f32,
    height: f32,
    short: f32,
    /// Where motifs along an edge are centred, from the edge.
    band: f32,
}

impl Frame {
    /// The four corners where the rules meet, each with the direction into the page.
    fn corners(&self) -> [(Point, Point); 4] {
        let inset: f32 = self.short * INNER;
        let (right, bottom): (f32, f32) = (self.width - inset, self.height - inset);
        [
            (Point::new(inset, inset), Point::new(1.0, 1.0)),
            (Point::new(right, inset), Point::new(-1.0, 1.0)),
            (Point::new(inset, bottom), Point::new(1.0, -1.0)),
            (Point::new(right, bottom), Point::new(-1.0, -1.0)),
        ]
    }

    /// Points along every edge's band every `spacing` pixels, leaving `corner` pixels clear
    /// at the ends, with the edge's direction in radians.
    fn along_edges(&self, spacing: f32, corner: f32) -> Vec<(Point, f32)> {
        let mut points: Vec<(Point, f32)> = Vec::new();
        let band: f32 = self.band;
        let edges: [(Point, Point); 4] = [
            (Point::new(0.0, band), Point::new(self.width, band)),
            (
                Point::new(0.0, self.height - band),
                Point::new(self.width, self.height - band),
            ),
            (Point::new(band, 0.0), Point::new(band, self.height)),
            (
                Point::new(self.width - band, 0.0),
                Point::new(self.width - band, self.height),
            ),
        ];
        for (start, end) in edges {
            let length: f32 = start.distance(end) - 2.0 * corner;
            if length <= 0.0 {
                continue;
            }
            let count: usize = (length / spacing) as usize + 1;
            let step: f32 = length / count as f32;
            let angle: f32 = libm::atan2f(end.y - start.y, end.x - start.x);
            let direction: Point = Point::at_angle(angle);
            for index in 0..=count {
                let at: Point = start + direction * (corner + step * index as f32);
                points.push((at, angle));
            }
        }
        points
    }
}

impl SeasonBorder {
    /// Returns the marks of `theme`'s border for a page of `resolution`.
    #[must_use]
    pub fn layout(theme: BorderTheme, resolution: Resolution) -> Vec<PenMark> {
        let width: f32 = resolution.width as f32;
        let height: f32 = resolution.height as f32;
        let short: f32 = width.min(height);
        let frame = Frame {
            width,
            height,
            short,
            band: short * INNER,
        };
        let mut pen = Engraver::new(short);
        let rule: Ink = match theme {
            BorderTheme::Frost => Ink::Blue,
            BorderTheme::Christmas => Ink::Red,
            BorderTheme::Blossom | BorderTheme::Midsummer | BorderTheme::Meadow => Ink::Green,
            BorderTheme::Autumn | BorderTheme::Easter => Ink::Black,
        };
        rules(&mut pen, &frame, rule);
        match theme {
            BorderTheme::Frost => frost(&mut pen, &frame),
            BorderTheme::Blossom => blossom(&mut pen, &frame),
            BorderTheme::Meadow => meadow(&mut pen, &frame),
            BorderTheme::Autumn => autumn(&mut pen, &frame),
            BorderTheme::Christmas => christmas(&mut pen, &frame),
            BorderTheme::Easter => easter(&mut pen, &frame),
            BorderTheme::Midsummer => midsummer(&mut pen, &frame),
        }
        pen.marks
    }
}

/// A number from -1 to 1 for `index`, the same every time, for a hand-drawn irregularity.
fn wobble(index: usize, salt: u32) -> f32 {
    let mut hash: u32 = (index as u32).wrapping_mul(0x9e37_79b9) ^ salt.wrapping_mul(0x85eb_ca6b);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2c1b_3c6d);
    hash ^= hash >> 12;
    (hash % 2001) as f32 / 1000.0 - 1.0
}

/// The double rule: a black hairline outside, a finer one in the theme's ink inside.
fn rules(pen: &mut Engraver, frame: &Frame, ink: Ink) {
    for (inset, units, colour) in [(OUTER, 1.6, Ink::Black), (INNER, 1.1, ink)] {
        let inset: f32 = frame.short * inset;
        let corners: [Point; 4] = [
            Point::new(inset, inset),
            Point::new(frame.width - inset, inset),
            Point::new(frame.width - inset, frame.height - inset),
            Point::new(inset, frame.height - inset),
        ];
        for (index, &from) in corners.iter().enumerate() {
            pen.line(from, corners[(index + 1) % 4], units, colour);
        }
    }
}

/// The angle of `direction`.
fn angle_of(direction: Point) -> f32 {
    libm::atan2f(direction.y, direction.x)
}

fn frost(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    for (corner, inward) in frame.corners() {
        // Ferns creep from each corner along both edges, as frost grows on a window.
        let along_x: f32 = angle_of(Point::new(inward.x, 0.0)) + inward.x * inward.y * 0.05;
        let along_y: f32 = angle_of(Point::new(0.0, inward.y)) - inward.x * inward.y * 0.05;
        // Clockwise from the stem is inwards when the corner's two signs differ.
        let clockwise: f32 = -inward.x * inward.y;
        pen.ice_fern(corner, along_x, short * 0.24, (3, -clockwise), Ink::Blue);
        pen.ice_fern(corner, along_y, short * 0.24, (3, clockwise), Ink::Blue);
        pen.ice_fern(corner, angle_of(inward), short * 0.1, (2, 0.0), Ink::Blue);
        let flake: Point = corner + inward * (short * 0.02);
        pen.snowflake(flake, short * 0.04, 0.2, Ink::Blue);
    }
    for (index, (at, _)) in frame
        .along_edges(short * 0.085, short * 0.3)
        .into_iter()
        .enumerate()
    {
        let radius: f32 = short * if index % 2 == 0 { 0.017 } else { 0.011 };
        pen.snowflake(at, radius, wobble(index, 2), Ink::Blue);
    }
}

fn blossom(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    let corners: [(Point, Point); 4] = frame.corners();
    // A branch from two opposite corners, along the long way of the page.
    for (corner, inward) in [corners[0], corners[3]] {
        let landscape: bool = frame.width >= frame.height;
        let along: Point = if landscape {
            Point::new(inward.x, 0.0)
        } else {
            Point::new(0.0, inward.y)
        };
        let across: Point = Point::new(inward.x - along.x, inward.y - along.y);
        let length: f32 = short * 0.55;
        let end: Point = corner + along * length + across * (short * 0.006);
        let control: Point = corner + along * (length * 0.45) + across * (short * 0.03);
        let start: Point = corner;
        pen.curve(start, control, end, (4.5, 1.2), Ink::Black);
        for (index, t) in [0.16_f32, 0.33, 0.5, 0.67, 0.83, 0.98]
            .into_iter()
            .enumerate()
        {
            let at: Point = engraver::quadratic(start, control, end, t);
            // Leaves point into the page, never off it.
            let leaf_angle: f32 = if index % 2 == 0 {
                angle_of(along + across * 1.2)
            } else {
                angle_of(along * -0.5 + across)
            };
            pen.leaf(at, leaf_angle, short * 0.04, short * 0.018, Ink::Green);
            let twig_end: Point = at + across * (short * 0.022) + along * (short * 0.014);
            pen.line(at, twig_end, 1.6, Ink::Black);
            let radius: f32 = short * (0.024 - 0.004 * t);
            pen.flower(twig_end, radius, wobble(index, 3), Ink::Red, Ink::Black);
        }
        let bud: Point = engraver::quadratic(start, control, end, 0.6) - across * (short * 0.01);
        pen.leaf(
            bud,
            angle_of(along * -1.0 + across),
            short * 0.018,
            short * 0.011,
            Ink::Red,
        );
    }
    // A single blossom in the other two corners.
    for (corner, inward) in [corners[1], corners[2]] {
        let at: Point = corner + inward * (short * 0.012);
        pen.leaf(
            at,
            angle_of(inward) + 0.6,
            short * 0.03,
            short * 0.014,
            Ink::Green,
        );
        pen.leaf(
            at,
            angle_of(inward) - 0.6,
            short * 0.03,
            short * 0.014,
            Ink::Green,
        );
        pen.flower(at, short * 0.02, 0.4, Ink::Red, Ink::Black);
    }
    // Petals drifting along the edges.
    for (index, (at, angle)) in frame
        .along_edges(short * 0.13, short * 0.62)
        .into_iter()
        .enumerate()
    {
        let turn: f32 = angle + 2.0 * wobble(index, 4);
        pen.leaf(at, turn, short * 0.016, short * 0.014, Ink::Red);
    }
}

fn meadow(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    let ground: f32 = frame.height - short * INNER;
    // Grasses and flowers along the foot, thickest in the corners.
    let blades: usize = (frame.width / (short * 0.008)) as usize;
    for index in 0..blades {
        let x: f32 = short * 0.03 + (frame.width - short * 0.06) * index as f32 / blades as f32;
        let from_edge: f32 = x.min(frame.width - x) / frame.width;
        let height: f32 =
            short * (0.05 - from_edge * 0.08).max(0.014) * (1.0 + 0.2 * wobble(index, 5));
        let lean: f32 = short * 0.012 * wobble(index, 6);
        let root: Point = Point::new(x, ground);
        let tip: Point = Point::new(x + lean * 1.6, ground - height);
        let control: Point = Point::new(x + lean * 0.2, ground - height * 0.6);
        pen.curve(root, control, tip, (1.6, 0.8), Ink::Green);
    }
    for (index, fraction) in [0.05_f32, 0.12, 0.2, 0.8, 0.88, 0.95]
        .into_iter()
        .enumerate()
    {
        let x: f32 = frame.width * fraction;
        let height: f32 = short * (0.045 + 0.006 * wobble(index, 7));
        let head: Point = Point::new(x + short * 0.006 * wobble(index, 8), ground - height);
        pen.curve(
            Point::new(x, ground),
            Point::new(x, ground - height * 0.5),
            head,
            (1.3, 1.0),
            Ink::Green,
        );
        let radius: f32 = short * 0.017;
        match index % 3 {
            0 => pen.daisy(head, radius, wobble(index, 9)),
            1 => pen.cornflower(head, radius, wobble(index, 9)),
            _ => pen.flower(head, radius, wobble(index, 9), Ink::Red, Ink::Black),
        }
    }
    // A butterfly in each top corner.
    for (index, (corner, inward)) in frame.corners()[..2].iter().enumerate() {
        let wings: Ink = if index == 0 { Ink::Yellow } else { Ink::Blue };
        butterfly(
            pen,
            *corner + *inward * (short * 0.03),
            short * 0.026,
            wings,
        );
    }
}

fn autumn(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    let inks: [Ink; 5] = [Ink::Red, Ink::Yellow, Ink::Red, Ink::Green, Ink::Yellow];
    for (index, (at, angle)) in frame
        .along_edges(short * 0.07, short * 0.12)
        .into_iter()
        .enumerate()
    {
        let turn: f32 = angle + PI * 0.25 + 1.4 * wobble(index, 11);
        let length: f32 = short * (0.028 + 0.004 * wobble(index, 12));
        let base: Point = at - Point::at_angle(turn) * (length * 0.5);
        pen.leaf(base, turn, length, length * 0.5, inks[index % inks.len()]);
    }
    for (corner_index, (corner, inward)) in frame.corners().into_iter().enumerate() {
        let towards: f32 = angle_of(inward);
        for (leaf, spread) in [-0.7_f32, 0.0, 0.7].into_iter().enumerate() {
            let ink: Ink = inks[(corner_index + leaf) % inks.len()];
            pen.leaf(corner, towards + spread, short * 0.065, short * 0.03, ink);
        }
        // An acorn in its cup.
        let acorn: Point =
            corner + inward * (short * 0.04) + Point::new(inward.x, 0.0) * (short * 0.04);
        pen.hatched_ellipse(acorn, (short * 0.009, short * 0.014), towards, Ink::Yellow);
        pen.hatched_ellipse(
            acorn - Point::at_angle(towards) * (short * 0.011),
            (short * 0.011, short * 0.005),
            towards,
            Ink::Green,
        );
    }
}

/// A butterfly with `wings` hatched in their ink, seen from above.
fn butterfly(pen: &mut Engraver, body: Point, size: f32, wings: Ink) {
    for (angle, length, width) in [
        (-2.3_f32, 1.0_f32, 0.75_f32),
        (-0.84, 1.0, 0.75),
        (2.5, 0.7, 0.55),
        (0.64, 0.7, 0.55),
    ] {
        pen.blade(body, angle, size * length, size * width, wings, Ink::Black);
    }
    pen.line(
        body + Point::new(0.0, -size * 0.35),
        body + Point::new(0.0, size * 0.45),
        2.6,
        Ink::Black,
    );
    for side in [-1.0_f32, 1.0] {
        let tip: Point = body + Point::new(side * size * 0.3, -size * 0.75);
        pen.curve(
            body + Point::new(0.0, -size * 0.3),
            body + Point::new(side * size * 0.05, -size * 0.7),
            tip,
            (1.0, 1.0),
            Ink::Black,
        );
        pen.dot(tip, size * 0.05, Ink::Black);
    }
}

/// A red berry outlined in black with a speck of light.
fn berry(pen: &mut Engraver, at: Point, radius: f32) {
    pen.dot(at, radius, Ink::Red);
    pen.ring(at, radius, 1.1, Ink::Black);
    pen.dot(
        at + Point::new(-radius * 0.35, -radius * 0.35),
        radius * 0.25,
        Ink::White,
    );
}

fn christmas(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    for (corner, inward) in frame.corners() {
        let towards: f32 = angle_of(inward);
        let length: f32 = short * 0.08;
        for spread in [-0.75_f32, 0.75] {
            pen.holly(corner, towards + spread, length, length * 0.45);
        }
        pen.holly(corner, towards, length * 0.85, length * 0.42);
        for (index, offset) in [(-0.6_f32, 0.6_f32), (0.6, 0.6), (0.0, -0.1)]
            .into_iter()
            .enumerate()
        {
            let at: Point = corner
                + inward * (short * 0.012)
                + Point::new(offset.0 * inward.x, offset.1 * inward.y) * (short * 0.012);
            berry(pen, at, short * (0.0085 + 0.001 * wobble(index, 13)));
        }
    }
    for (index, (at, angle)) in frame
        .along_edges(short * 0.11, short * 0.2)
        .into_iter()
        .enumerate()
    {
        let side: Point = Point::at_angle(angle) * (short * 0.006);
        pen.holly(at - side, angle + PI + 0.35, short * 0.032, short * 0.014);
        pen.holly(at + side, angle - 0.35, short * 0.032, short * 0.014);
        berry(pen, at, short * (0.008 + 0.001 * wobble(index, 14)));
    }
}

fn easter(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    let corners: [(Point, Point); 4] = frame.corners();
    let feathers: [Ink; 4] = [Ink::Yellow, Ink::Red, Ink::Blue, Ink::Green];
    // Birch twigs from the top corners, along the top and down the side, hung with feathers.
    for (corner, inward) in &corners[..2] {
        for (twig, direction) in [Point::new(inward.x, 0.0), Point::new(0.0, inward.y)]
            .into_iter()
            .enumerate()
        {
            let length: f32 = short * 0.3;
            let across: Point = Point::new(inward.x - direction.x, inward.y - direction.y);
            let end: Point = *corner + direction * length + across * (short * 0.008);
            let control: Point = *corner + direction * (length * 0.5) + across * (short * 0.018);
            pen.curve(*corner, control, end, (2.4, 0.8), Ink::Black);
            for (index, t) in [0.2_f32, 0.38, 0.56, 0.74, 0.92].into_iter().enumerate() {
                let at: Point = engraver::quadratic(*corner, control, end, t);
                let shoot: Point = at + direction * (short * 0.02) + across * (short * 0.012);
                pen.line(at, shoot, 1.0, Ink::Black);
                let ink: Ink = feathers[(index + twig) % feathers.len()];
                let hang: f32 = FRAC_PI_2 + 0.25 * wobble(index + twig * 5, 15);
                pen.feather(at, hang, short * 0.05, ink);
            }
        }
    }
    // Eggs in the bottom corners, on a tuft of grass.
    let eggs: [Ink; 3] = [Ink::Yellow, Ink::Red, Ink::Blue];
    for (corner, inward) in &corners[2..] {
        for blade in 0..9 {
            let x: f32 = corner.x + inward.x * short * (0.005 + 0.011 * blade as f32);
            let height: f32 = short * (0.02 + 0.006 * wobble(blade, 16));
            let root: Point = Point::new(x, corner.y);
            let tip: Point = Point::new(x + short * 0.006 * wobble(blade, 17), corner.y - height);
            pen.curve(root, root.lerp(tip, 0.6), tip, (1.4, 0.8), Ink::Green);
        }
        for (index, ink) in eggs.into_iter().enumerate() {
            let radii: (f32, f32) = (short * 0.012, short * 0.016);
            let x: f32 = corner.x + inward.x * short * (0.018 + 0.026 * index as f32);
            let centre: Point = Point::new(x, corner.y - radii.1 * 0.95);
            pen.hatched_ellipse(centre, radii, -FRAC_PI_2 + 0.15 * wobble(index, 18), ink);
            pen.line(
                centre + Point::new(-radii.0 * 0.85, 0.0),
                centre + Point::new(radii.0 * 0.85, 0.0),
                1.6,
                Ink::White,
            );
        }
    }
}

fn midsummer(pen: &mut Engraver, frame: &Frame) {
    let short: f32 = frame.short;
    let band: f32 = frame.band;
    // A garland of birch leaves and flowers along the top and the foot.
    for (row, y) in [band, frame.height - band].into_iter().enumerate() {
        let spacing: f32 = short * 0.06;
        let count: usize = ((frame.width - short * 0.06) / spacing) as usize;
        for index in 0..=count {
            let x: f32 = short * 0.03 + (frame.width - short * 0.06) * index as f32 / count as f32;
            let at: Point = Point::new(x, y);
            let sway: f32 = 0.5 * wobble(index, 20 + row as u32);
            // Leaves droop into the page: down from the top, up from the foot.
            let into: f32 = if row == 0 { 1.0 } else { -1.0 };
            pen.leaf(
                at,
                into * PI * 0.85 + sway,
                short * 0.028,
                short * 0.014,
                Ink::Green,
            );
            pen.leaf(
                at,
                into * PI * 0.15 + sway,
                short * 0.028,
                short * 0.014,
                Ink::Green,
            );
            let radius: f32 = short * 0.014;
            match index % 4 {
                0 => pen.daisy(at, radius, sway),
                1 => pen.cornflower(at, radius, sway),
                2 => pen.flower(at, radius, sway, Ink::Yellow, Ink::Black),
                _ => pen.flower(at, radius, sway, Ink::Red, Ink::Black),
            }
        }
    }
    // Birch sprigs down the sides.
    for (corner, inward) in frame.corners() {
        for leaf in 0..3 {
            let at: Point =
                corner + Point::new(0.0, inward.y) * (short * (0.04 + 0.03 * leaf as f32));
            let angle: f32 = angle_of(Point::new(inward.x, inward.y * 0.6));
            pen.leaf(at, angle, short * 0.028, short * 0.014, Ink::Green);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn bounds(marks: &[PenMark]) -> (f32, f32, f32, f32) {
        let mut bounds: (f32, f32, f32, f32) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut grow = |point: Point, reach: f32| {
            bounds.0 = bounds.0.min(point.x - reach);
            bounds.1 = bounds.1.min(point.y - reach);
            bounds.2 = bounds.2.max(point.x + reach);
            bounds.3 = bounds.3.max(point.y + reach);
        };
        for mark in marks {
            match *mark {
                PenMark::Line {
                    from, to, width, ..
                } => {
                    grow(from, width / 2.0);
                    grow(to, width / 2.0);
                }
                PenMark::Dot { centre, radius, .. } => grow(centre, radius),
            }
        }
        bounds
    }

    #[test]
    fn every_theme_stays_on_the_page_and_off_its_middle() {
        for resolution in [
            Resolution::new(1200, 1600),
            Resolution::new(1600, 1200),
            Resolution::new(800, 480),
        ] {
            let (width, height) = (resolution.width as f32, resolution.height as f32);
            let short: f32 = width.min(height);
            for theme in BorderTheme::ALL {
                let marks: Vec<PenMark> = SeasonBorder::layout(theme, resolution);
                assert!(marks.len() > 40, "{theme} has {} marks", marks.len());
                let (left, top, right, bottom) = bounds(&marks);
                assert!(left >= -1.0 && top >= -1.0, "{theme}: {left}, {top}");
                assert!(right <= width + 1.0 && bottom <= height + 1.0, "{theme}");
                // Nothing reaches the middle: the ornament is a border.
                let inner = (
                    short * 0.2,
                    short * 0.2,
                    width - short * 0.2,
                    height - short * 0.2,
                );
                let in_middle = marks.iter().any(|mark| match *mark {
                    PenMark::Line { from, .. } | PenMark::Dot { centre: from, .. } => {
                        from.x > inner.0 && from.x < inner.2 && from.y > inner.1 && from.y < inner.3
                    }
                });
                assert!(
                    !in_middle,
                    "{theme} reaches into the page at {resolution:?}"
                );
            }
        }
    }

    #[test]
    fn uses_the_theme_inks() {
        let inks = |theme: BorderTheme| -> Vec<Ink> {
            let mut inks: Vec<Ink> = Vec::new();
            for mark in SeasonBorder::layout(theme, Resolution::new(1200, 1600)) {
                let (PenMark::Line { ink, .. } | PenMark::Dot { ink, .. }) = mark;
                if !inks.contains(&ink) {
                    inks.push(ink);
                }
            }
            inks
        };
        assert!(inks(BorderTheme::Frost).contains(&Ink::Blue));
        assert!(!inks(BorderTheme::Frost).contains(&Ink::Red));
        for ink in [Ink::Red, Ink::Green, Ink::Black] {
            assert!(inks(BorderTheme::Christmas).contains(&ink));
        }
        assert_eq!(inks(BorderTheme::Easter).len(), 6);
    }
}
