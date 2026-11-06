//! Which shape and colours each species gets.

use frame_core::Rgb;

use crate::build::{self, Build};
use crate::plumage::{LEG, Plumage, SNOW, SOOT};

/// A species with its own look.
struct Known {
    scientific_name: &'static str,
    build: Build,
    plumage: Plumage,
}

const fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb::new(red, green, blue)
}

const YELLOW: Rgb = rgb(244, 214, 58);
const OLIVE: Rgb = rgb(118, 138, 66);
const BLACK: Rgb = rgb(30, 30, 32);

/// Garden birds of northern Europe: the mock station's residents and a few more.
const KNOWN: [Known; 18] = [
    Known {
        scientific_name: "Parus major",
        build: build::TIT,
        plumage: Plumage {
            head: BLACK,
            wing: rgb(96, 118, 140),
            tail: rgb(80, 96, 112),
            cheek: Some(SNOW),
            bib: Some(BLACK),
            wing_bar: Some(SNOW),
            ..Plumage::plain(OLIVE, YELLOW)
        },
    },
    Known {
        scientific_name: "Cyanistes caeruleus",
        build: build::TIT,
        plumage: Plumage {
            head: SNOW,
            cap: Some(rgb(58, 112, 206)),
            eye_stripe: Some(rgb(30, 40, 80)),
            wing: rgb(58, 112, 206),
            tail: rgb(58, 100, 180),
            wing_bar: Some(SNOW),
            ..Plumage::plain(rgb(132, 156, 82), YELLOW)
        },
    },
    Known {
        scientific_name: "Erithacus rubecula",
        build: build::THRUSH,
        plumage: Plumage {
            breast: Some(rgb(232, 112, 46)),
            face: Some(rgb(232, 112, 46)),
            ..Plumage::plain(rgb(128, 108, 74), rgb(238, 232, 220))
        },
    },
    Known {
        scientific_name: "Turdus merula",
        build: build::THRUSH,
        plumage: Plumage {
            beak: rgb(246, 170, 30),
            eye_ring: Some(rgb(246, 190, 40)),
            legs: rgb(70, 56, 46),
            ..Plumage::plain(BLACK, rgb(44, 42, 42))
        },
    },
    Known {
        scientific_name: "Fringilla coelebs",
        build: build::FINCH,
        plumage: Plumage {
            head: rgb(204, 132, 112),
            cap: Some(rgb(104, 124, 156)),
            nape: Some(rgb(104, 124, 156)),
            breast: Some(rgb(206, 128, 108)),
            wing: rgb(52, 48, 46),
            wing_bar: Some(SNOW),
            shoulder: Some(SNOW),
            beak: rgb(150, 160, 170),
            ..Plumage::plain(rgb(150, 104, 72), rgb(226, 190, 176))
        },
    },
    Known {
        scientific_name: "Passer domesticus",
        build: build::FINCH,
        plumage: Plumage {
            head: rgb(214, 208, 200),
            cap: Some(rgb(128, 128, 134)),
            nape: Some(rgb(150, 92, 54)),
            bib: Some(BLACK),
            wing: rgb(130, 88, 56),
            wing_bar: Some(SNOW),
            ..Plumage::plain(rgb(156, 112, 70), rgb(204, 200, 190))
        },
    },
    Known {
        scientific_name: "Pica pica",
        build: build::MAGPIE,
        plumage: Plumage {
            shoulder: Some(SNOW),
            tail: rgb(36, 50, 70),
            ..Plumage::plain(BLACK, SNOW)
        },
    },
    Known {
        scientific_name: "Corvus cornix",
        build: build::CROW,
        plumage: Plumage {
            head: BLACK,
            bib: Some(BLACK),
            wing: BLACK,
            tail: BLACK,
            legs: BLACK,
            ..Plumage::plain(rgb(156, 154, 152), rgb(170, 168, 166))
        },
    },
    Known {
        scientific_name: "Troglodytes troglodytes",
        build: build::WREN,
        plumage: Plumage {
            eye_stripe: Some(rgb(214, 184, 140)),
            wing_bar: Some(rgb(96, 64, 40)),
            ..Plumage::plain(rgb(146, 100, 62), rgb(196, 156, 114))
        },
    },
    Known {
        scientific_name: "Turdus philomelos",
        build: build::THRUSH,
        plumage: Plumage {
            spots: Some(rgb(76, 54, 38)),
            breast: Some(rgb(240, 214, 160)),
            ..Plumage::plain(rgb(140, 110, 76), rgb(240, 232, 206))
        },
    },
    Known {
        scientific_name: "Chloris chloris",
        build: build::FINCH,
        plumage: Plumage {
            wing: rgb(96, 120, 54),
            wing_bar: Some(YELLOW),
            beak: rgb(222, 186, 166),
            ..Plumage::plain(rgb(118, 150, 58), rgb(176, 196, 84))
        },
    },
    Known {
        scientific_name: "Carduelis carduelis",
        build: build::FINCH,
        plumage: Plumage {
            head: SNOW,
            face: Some(rgb(212, 38, 40)),
            cap: Some(BLACK),
            nape: Some(BLACK),
            wing: BLACK,
            wing_bar: Some(YELLOW),
            tail: BLACK,
            beak: rgb(232, 210, 190),
            ..Plumage::plain(rgb(176, 134, 94), rgb(240, 226, 206))
        },
    },
    Known {
        scientific_name: "Dendrocopos major",
        build: build::WOODPECKER,
        plumage: Plumage {
            head: SNOW,
            cap: Some(BLACK),
            nape: Some(rgb(214, 36, 40)),
            eye_stripe: Some(BLACK),
            shoulder: Some(SNOW),
            vent: Some(rgb(214, 36, 40)),
            wing_bar: Some(SNOW),
            ..Plumage::plain(BLACK, rgb(236, 230, 220))
        },
    },
    Known {
        scientific_name: "Sitta europaea",
        build: build::NUTHATCH,
        plumage: Plumage {
            head: SNOW,
            cap: Some(rgb(104, 124, 156)),
            eye_stripe: Some(BLACK),
            vent: Some(rgb(190, 96, 52)),
            ..Plumage::plain(rgb(104, 124, 156), rgb(226, 160, 104))
        },
    },
    Known {
        scientific_name: "Aegithalos caudatus",
        build: build::LONG_TAILED_TIT,
        plumage: Plumage {
            head: SNOW,
            eye_stripe: Some(BLACK),
            shoulder: Some(rgb(216, 150, 160)),
            wing: BLACK,
            tail: BLACK,
            vent: Some(rgb(222, 170, 176)),
            ..Plumage::plain(rgb(60, 56, 58), rgb(240, 230, 230))
        },
    },
    Known {
        scientific_name: "Columba palumbus",
        build: build::PIGEON,
        plumage: Plumage {
            breast: Some(rgb(184, 142, 156)),
            nape: Some(SNOW),
            wing: rgb(126, 132, 150),
            wing_bar: Some(SNOW),
            tail: rgb(90, 94, 110),
            beak: rgb(236, 190, 70),
            legs: rgb(200, 100, 110),
            ..Plumage::plain(rgb(140, 146, 164), rgb(170, 172, 186))
        },
    },
    Known {
        scientific_name: "Pyrrhula pyrrhula",
        build: build::FINCH,
        plumage: Plumage {
            head: BLACK,
            breast: Some(rgb(222, 88, 76)),
            cheek: Some(rgb(222, 88, 76)),
            wing: BLACK,
            wing_bar: Some(rgb(220, 220, 220)),
            tail: BLACK,
            ..Plumage::plain(rgb(130, 130, 140), rgb(222, 88, 76))
        },
    },
    Known {
        scientific_name: "Regulus regulus",
        build: build::TIT,
        plumage: Plumage {
            crown_stripe: Some(rgb(250, 180, 30)),
            cap: Some(rgb(110, 128, 64)),
            wing: rgb(90, 104, 56),
            wing_bar: Some(SNOW),
            legs: LEG,
            ..Plumage::plain(rgb(134, 152, 84), rgb(216, 222, 176))
        },
    },
];

/// Returns the shape and colours for a species: its own if it has one, otherwise ones derived
/// from its name, the same every time.
pub(crate) fn look(scientific_name: &str) -> (Build, Plumage) {
    if let Some(known) = KNOWN
        .iter()
        .find(|known| same_name(known.scientific_name, scientific_name))
    {
        return (known.build, known.plumage);
    }
    derived(name_hash(scientific_name))
}

fn same_name(left: &str, right: &str) -> bool {
    let mut left_words = left.split_ascii_whitespace();
    let mut right_words = right.split_ascii_whitespace();
    loop {
        match (left_words.next(), right_words.next()) {
            (None, None) => return true,
            (Some(left_word), Some(right_word)) if left_word.eq_ignore_ascii_case(right_word) => {}
            _ => return false,
        }
    }
}

/// FNV-1a over the lowercase letters of the name, so spelling variants agree.
fn name_hash(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        hash ^= u64::from(byte.to_ascii_lowercase());
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// A plausible songbird from a hash: a shape, a hue for the back, a pale belly, and maybe a cap,
/// a breast patch and wing bars.
fn derived(hash: u64) -> (Build, Plumage) {
    let pick = |shift: u32, choices: u64| -> u64 { (hash >> shift) % choices };
    let build: Build = build::COMMON[usize::try_from(pick(0, 4)).unwrap_or(0)];
    let hue: f32 = pick(8, 360) as f32;
    let back: Rgb = from_hsv(hue, 0.45, 0.6);
    let belly: Rgb = from_hsv(hue + 30.0, 0.2, 0.93);
    let accent: Rgb = from_hsv(hue + 150.0 + pick(20, 60) as f32, 0.75, 0.85);
    let plumage = Plumage {
        wing: from_hsv(hue, 0.45, 0.45),
        tail: from_hsv(hue, 0.45, 0.4),
        cap: (pick(28, 3) == 0).then_some(BLACK),
        breast: (pick(32, 2) == 0).then_some(accent),
        wing_bar: (pick(36, 2) == 0).then_some(SNOW),
        cheek: (pick(40, 3) == 0).then_some(SNOW),
        beak: if pick(44, 2) == 0 {
            SOOT
        } else {
            rgb(200, 150, 90)
        },
        ..Plumage::plain(back, belly)
    };
    (build, plumage)
}

/// Converts hue (degrees), saturation and value (0 to 1) to a colour.
fn from_hsv(hue: f32, saturation: f32, value: f32) -> Rgb {
    let hue: f32 = wrap(hue, 360.0) / 60.0;
    let chroma: f32 = value * saturation;
    let second: f32 = chroma * (1.0 - libm::fabsf(wrap(hue, 2.0) - 1.0));
    let (red, green, blue): (f32, f32, f32) = match hue as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let lift: f32 = value - chroma;
    let byte = |channel: f32| -> u8 { ((channel + lift) * 255.0 + 0.5).clamp(0.0, 255.0) as u8 };
    Rgb::new(byte(red), byte(green), byte(blue))
}

/// Returns `value` modulo `period`, in `0..period`.
fn wrap(value: f32, period: f32) -> f32 {
    let remainder: f32 = libm::fmodf(value, period);
    if remainder < 0.0 {
        remainder + period
    } else {
        remainder
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known_species_ignore_case_and_spacing() {
        assert_eq!(look("parus  MAJOR").1.head, BLACK);
        assert_ne!(look("Parusmajor").1.head, BLACK);
    }

    #[test]
    fn unknown_species_are_stable_and_varied() {
        assert_eq!(look("Strix aluco"), look("strix aluco"));
        assert_ne!(look("Strix aluco").1.back, look("Bubo bubo").1.back);
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(from_hsv(0.0, 1.0, 1.0), Rgb::new(255, 0, 0));
        assert_eq!(from_hsv(120.0, 1.0, 1.0), Rgb::new(0, 255, 0));
        assert_eq!(from_hsv(240.0, 1.0, 1.0), Rgb::new(0, 0, 255));
        assert_eq!(from_hsv(0.0, 0.0, 1.0), Rgb::WHITE);
    }
}
