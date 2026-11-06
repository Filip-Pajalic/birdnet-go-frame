//! A species heard for the first time this year.

use birdnet_client::CivilDate;
use birdnet_model::SpeciesSummary;

/// A species and the day it was first heard this year.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    /// The species, with its count so far this year.
    pub species: SpeciesSummary,
    /// The day it was first heard.
    pub date: CivilDate,
}
