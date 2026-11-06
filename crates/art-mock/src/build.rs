//! Body shapes.

/// The proportions of a bird, in drawing units: the body is one unit long.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Build {
    /// Body depth as a fraction of its length.
    pub(crate) body_depth: f32,
    /// How far the front of the body is raised, in degrees.
    pub(crate) tilt_degrees: f32,
    pub(crate) head_radius: f32,
    pub(crate) beak_length: f32,
    /// Half the beak's depth where it meets the head.
    pub(crate) beak_depth: f32,
    pub(crate) tail_length: f32,
    pub(crate) tail_width: f32,
    /// Degrees below horizontal the tail points; negative cocks it up.
    pub(crate) tail_droop_degrees: f32,
    pub(crate) leg_length: f32,
}

/// Tits, and the default songbird.
pub(crate) const TIT: Build = Build {
    body_depth: 0.66,
    tilt_degrees: 18.0,
    head_radius: 0.25,
    beak_length: 0.09,
    beak_depth: 0.045,
    tail_length: 0.42,
    tail_width: 0.17,
    tail_droop_degrees: 28.0,
    leg_length: 0.2,
};

/// Finches and sparrows: a thick, conical beak.
pub(crate) const FINCH: Build = Build {
    body_depth: 0.64,
    tilt_degrees: 20.0,
    head_radius: 0.25,
    beak_length: 0.14,
    beak_depth: 0.085,
    tail_length: 0.44,
    tail_width: 0.17,
    tail_droop_degrees: 32.0,
    leg_length: 0.2,
};

/// Thrushes and robins: upright, long-legged.
pub(crate) const THRUSH: Build = Build {
    body_depth: 0.6,
    tilt_degrees: 28.0,
    head_radius: 0.22,
    beak_length: 0.16,
    beak_depth: 0.045,
    tail_length: 0.5,
    tail_width: 0.17,
    tail_droop_degrees: 38.0,
    leg_length: 0.3,
};

/// Crows: heavy beak, strong legs.
pub(crate) const CROW: Build = Build {
    body_depth: 0.56,
    tilt_degrees: 22.0,
    head_radius: 0.21,
    beak_length: 0.3,
    beak_depth: 0.08,
    tail_length: 0.55,
    tail_width: 0.2,
    tail_droop_degrees: 30.0,
    leg_length: 0.32,
};

/// Magpies: a crow with a very long tail.
pub(crate) const MAGPIE: Build = Build {
    tail_length: 1.15,
    tail_width: 0.16,
    tail_droop_degrees: 22.0,
    beak_length: 0.24,
    ..CROW
};

/// Woodpeckers: upright against a trunk, stiff tail, chisel beak.
pub(crate) const WOODPECKER: Build = Build {
    body_depth: 0.5,
    tilt_degrees: 62.0,
    head_radius: 0.21,
    beak_length: 0.28,
    beak_depth: 0.05,
    tail_length: 0.45,
    tail_width: 0.16,
    tail_droop_degrees: 80.0,
    leg_length: 0.12,
};

/// Nuthatches: level, short tail, long beak.
pub(crate) const NUTHATCH: Build = Build {
    body_depth: 0.6,
    tilt_degrees: 4.0,
    head_radius: 0.24,
    beak_length: 0.22,
    beak_depth: 0.045,
    tail_length: 0.28,
    tail_width: 0.17,
    tail_droop_degrees: 12.0,
    leg_length: 0.18,
};

/// Pigeons: plump, small head, short beak.
pub(crate) const PIGEON: Build = Build {
    body_depth: 0.62,
    tilt_degrees: 14.0,
    head_radius: 0.17,
    beak_length: 0.1,
    beak_depth: 0.035,
    tail_length: 0.55,
    tail_width: 0.2,
    tail_droop_degrees: 18.0,
    leg_length: 0.16,
};

/// Wrens: round, tail cocked straight up.
pub(crate) const WREN: Build = Build {
    body_depth: 0.74,
    tilt_degrees: 12.0,
    head_radius: 0.25,
    beak_length: 0.16,
    beak_depth: 0.035,
    tail_length: 0.32,
    tail_width: 0.14,
    tail_droop_degrees: -55.0,
    leg_length: 0.22,
};

/// Long-tailed tits: a round ball with a tail longer than the body.
pub(crate) const LONG_TAILED_TIT: Build = Build {
    body_depth: 0.78,
    tilt_degrees: 10.0,
    head_radius: 0.27,
    beak_length: 0.06,
    beak_depth: 0.04,
    tail_length: 1.1,
    tail_width: 0.12,
    tail_droop_degrees: 14.0,
    leg_length: 0.16,
};

/// The shapes a species without its own entry can get.
pub(crate) const COMMON: [Build; 4] = [TIT, FINCH, THRUSH, NUTHATCH];
