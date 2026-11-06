//! The simulated garden: which birds live there and how often they call.

/// A species in the simulated garden.
pub(crate) struct Resident {
    pub(crate) scientific_name: &'static str,
    pub(crate) common_name: &'static str,
    /// Detections per hour at the peak of the dawn chorus.
    pub(crate) peak_per_hour: u32,
    /// Percent chance the species turns up on a given day.
    pub(crate) presence_percent: u32,
}

/// Common garden birds of northern Europe: residents first, then visitors that turn up on
/// some days only, so a day's page holds anything from a dozen to thirty-odd species.
pub(crate) const RESIDENTS: [Resident; 35] = [
    Resident {
        scientific_name: "Parus major",
        common_name: "Great Tit",
        peak_per_hour: 40,
        presence_percent: 100,
    },
    Resident {
        scientific_name: "Cyanistes caeruleus",
        common_name: "Eurasian Blue Tit",
        peak_per_hour: 25,
        presence_percent: 95,
    },
    Resident {
        scientific_name: "Erithacus rubecula",
        common_name: "European Robin",
        peak_per_hour: 30,
        presence_percent: 95,
    },
    Resident {
        scientific_name: "Turdus merula",
        common_name: "Eurasian Blackbird",
        peak_per_hour: 28,
        presence_percent: 95,
    },
    Resident {
        scientific_name: "Fringilla coelebs",
        common_name: "Common Chaffinch",
        peak_per_hour: 22,
        presence_percent: 85,
    },
    Resident {
        scientific_name: "Passer domesticus",
        common_name: "House Sparrow",
        peak_per_hour: 35,
        presence_percent: 80,
    },
    Resident {
        scientific_name: "Pica pica",
        common_name: "Eurasian Magpie",
        peak_per_hour: 10,
        presence_percent: 90,
    },
    Resident {
        scientific_name: "Corvus cornix",
        common_name: "Hooded Crow",
        peak_per_hour: 8,
        presence_percent: 85,
    },
    Resident {
        scientific_name: "Troglodytes troglodytes",
        common_name: "Eurasian Wren",
        peak_per_hour: 12,
        presence_percent: 70,
    },
    Resident {
        scientific_name: "Turdus philomelos",
        common_name: "Song Thrush",
        peak_per_hour: 14,
        presence_percent: 55,
    },
    Resident {
        scientific_name: "Chloris chloris",
        common_name: "European Greenfinch",
        peak_per_hour: 9,
        presence_percent: 60,
    },
    Resident {
        scientific_name: "Carduelis carduelis",
        common_name: "European Goldfinch",
        peak_per_hour: 7,
        presence_percent: 50,
    },
    Resident {
        scientific_name: "Dendrocopos major",
        common_name: "Great Spotted Woodpecker",
        peak_per_hour: 4,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Sitta europaea",
        common_name: "Eurasian Nuthatch",
        peak_per_hour: 6,
        presence_percent: 40,
    },
    Resident {
        scientific_name: "Aegithalos caudatus",
        common_name: "Long-tailed Tit",
        peak_per_hour: 5,
        presence_percent: 35,
    },
    Resident {
        scientific_name: "Columba palumbus",
        common_name: "Common Wood Pigeon",
        peak_per_hour: 6,
        presence_percent: 75,
    },
    Resident {
        scientific_name: "Pyrrhula pyrrhula",
        common_name: "Eurasian Bullfinch",
        peak_per_hour: 3,
        presence_percent: 30,
    },
    Resident {
        scientific_name: "Regulus regulus",
        common_name: "Goldcrest",
        peak_per_hour: 4,
        presence_percent: 35,
    },
    Resident {
        scientific_name: "Spinus spinus",
        common_name: "Eurasian Siskin",
        peak_per_hour: 12,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Turdus pilaris",
        common_name: "Fieldfare",
        peak_per_hour: 9,
        presence_percent: 40,
    },
    Resident {
        scientific_name: "Turdus viscivorus",
        common_name: "Mistle Thrush",
        peak_per_hour: 6,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Turdus iliacus",
        common_name: "Redwing",
        peak_per_hour: 8,
        presence_percent: 35,
    },
    Resident {
        scientific_name: "Periparus ater",
        common_name: "Coal Tit",
        peak_per_hour: 14,
        presence_percent: 60,
    },
    Resident {
        scientific_name: "Certhia familiaris",
        common_name: "Eurasian Treecreeper",
        peak_per_hour: 6,
        presence_percent: 50,
    },
    Resident {
        scientific_name: "Passer montanus",
        common_name: "Eurasian Tree Sparrow",
        peak_per_hour: 15,
        presence_percent: 50,
    },
    Resident {
        scientific_name: "Coloeus monedula",
        common_name: "Western Jackdaw",
        peak_per_hour: 10,
        presence_percent: 60,
    },
    Resident {
        scientific_name: "Motacilla alba",
        common_name: "White Wagtail",
        peak_per_hour: 7,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Sturnus vulgaris",
        common_name: "Common Starling",
        peak_per_hour: 18,
        presence_percent: 60,
    },
    Resident {
        scientific_name: "Strix aluco",
        common_name: "Tawny Owl",
        peak_per_hour: 3,
        presence_percent: 40,
    },
    Resident {
        scientific_name: "Garrulus glandarius",
        common_name: "Eurasian Jay",
        peak_per_hour: 5,
        presence_percent: 55,
    },
    Resident {
        scientific_name: "Coccothraustes coccothraustes",
        common_name: "Hawfinch",
        peak_per_hour: 4,
        presence_percent: 30,
    },
    Resident {
        scientific_name: "Poecile palustris",
        common_name: "Marsh Tit",
        peak_per_hour: 8,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Muscicapa striata",
        common_name: "Spotted Flycatcher",
        peak_per_hour: 6,
        presence_percent: 35,
    },
    Resident {
        scientific_name: "Phylloscopus trochilus",
        common_name: "Willow Warbler",
        peak_per_hour: 14,
        presence_percent: 45,
    },
    Resident {
        scientific_name: "Fringilla montifringilla",
        common_name: "Brambling",
        peak_per_hour: 8,
        presence_percent: 30,
    },
];

/// Returns a well-mixed 64-bit value for `seed` (SplitMix64), the only randomness the mock uses.
pub(crate) fn mix(seed: u64) -> u64 {
    let mut value: u64 = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Returns the activity level, in percent of the peak, at `hour` (0 to 23, local).
pub(crate) fn activity_percent(hour: u32) -> u32 {
    match hour {
        0..=3 | 22..=23 => 0,
        4 => 20,
        5..=7 => 100,
        8..=10 => 60,
        11..=15 => 30,
        16..=19 => 45,
        _ => 15,
    }
}
