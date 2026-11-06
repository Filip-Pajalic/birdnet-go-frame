//! Types for the parts of BirdNET-Go's `/api/v2` the frame reads, and their JSON parsing.
//!
//! `no_std` with `alloc` and no dependencies (JSON is read by a small built-in reader), so the
//! ESP32 can parse the same responses as the server.

#![no_std]

extern crate alloc;

mod detection;
mod json;
mod parse_error;
mod scientific_name;
mod species_summary;

pub use detection::Detection;
pub use parse_error::ParseError;
pub use scientific_name::ScientificName;
pub use species_summary::SpeciesSummary;
