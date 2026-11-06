//! Feather colours.

use frame_core::Rgb;

/// A bird's colours. Optional patches are left out when `None`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plumage {
    pub(crate) back: Rgb,
    pub(crate) belly: Rgb,
    pub(crate) head: Rgb,
    pub(crate) wing: Rgb,
    pub(crate) tail: Rgb,
    pub(crate) beak: Rgb,
    pub(crate) legs: Rgb,
    pub(crate) eye_ring: Option<Rgb>,
    /// The front of the body, below the throat.
    pub(crate) breast: Option<Rgb>,
    /// Feathers under the tail.
    pub(crate) vent: Option<Rgb>,
    /// Spots over the belly.
    pub(crate) spots: Option<Rgb>,
    pub(crate) wing_bar: Option<Rgb>,
    /// A patch at the front of the wing.
    pub(crate) shoulder: Option<Rgb>,
    /// The top of the head.
    pub(crate) cap: Option<Rgb>,
    /// A stripe along the crown, as on a goldcrest.
    pub(crate) crown_stripe: Option<Rgb>,
    /// The back of the head.
    pub(crate) nape: Option<Rgb>,
    /// The front of the head around the beak.
    pub(crate) face: Option<Rgb>,
    pub(crate) cheek: Option<Rgb>,
    /// A band through the eye.
    pub(crate) eye_stripe: Option<Rgb>,
    /// A patch under the beak, down the throat.
    pub(crate) bib: Option<Rgb>,
}

/// Dark brown-black for beaks and stripes.
pub(crate) const SOOT: Rgb = Rgb::new(40, 36, 34);
/// Off-white feathers.
pub(crate) const SNOW: Rgb = Rgb::new(246, 244, 238);
/// Grey-pink legs.
pub(crate) const LEG: Rgb = Rgb::new(120, 105, 100);

impl Plumage {
    /// A plain bird: one colour above, another below, everything else derived.
    pub(crate) const fn plain(back: Rgb, belly: Rgb) -> Self {
        Self {
            back,
            belly,
            head: back,
            wing: back,
            tail: back,
            beak: SOOT,
            legs: LEG,
            eye_ring: None,
            breast: None,
            vent: None,
            spots: None,
            wing_bar: None,
            shoulder: None,
            cap: None,
            crown_stripe: None,
            nape: None,
            face: None,
            cheek: None,
            eye_stripe: None,
            bib: None,
        }
    }
}
