//! A few facts per bird species, for one line about the page's main bird.
//!
//! [`BirdFacts::bundled`] embeds a table built from AVONET (CC BY 4.0) and Wikidata (CC0) for
//! the species BirdNET can detect; see `data/ATTRIBUTION.md`. Nothing is looked up online.
//! [`BirdFacts::fact_of_the_day`] picks one [`Fact`] per species and day, so the line changes
//! daily but not between wakes. Words are the caller's: a fact is plain data. `no_std`, no
//! allocation.
//!
//! ```
//! use bird_facts::{BirdFacts, Fact};
//!
//! let facts = BirdFacts::bundled();
//! let great_tit: Vec<Fact> = facts.facts("Parus major").collect();
//! assert!(great_tit.contains(&Fact::Wingspan { centimetres: 23 }));
//! assert!(facts.fact_of_the_day("Parus major", 20_400).is_some());
//! assert_eq!(facts.fact_of_the_day("Not a bird", 20_400), None);
//! ```

#![no_std]

mod bird_facts;
mod fact;

pub use bird_facts::BirdFacts;
pub use fact::{Diet, Fact, Lifestyle, Migration};
