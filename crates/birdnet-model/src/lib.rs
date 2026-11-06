//! Types for the parts of BirdNET-Go's `/api/v2` the frame reads, and their JSON parsing.
//!
//! `no_std` with `alloc`, its JSON read by `lite-json`, so the ESP32 can parse the same responses
//! as the server.

#![no_std]

extern crate alloc;

mod detection;
mod parse_error;
mod scientific_name;
mod species_hours;
mod species_summary;

pub use detection::Detection;
pub use parse_error::ParseError;
pub use scientific_name::ScientificName;
pub use species_hours::SpeciesHours;
pub use species_summary::SpeciesSummary;
