//! A small JSON reader, `no_std` with `alloc` and no dependencies.
//!
//! [`Json::parse`] builds a tree, which suits documents of up to a few megabytes: BirdNET-Go's
//! responses, settings and manifest files. It replaces serde and serde_json (and the proc-macro
//! crates they bring) for this project's needs.
//!
//! ```
//! use lite_json::Json;
//!
//! let value = Json::parse(br#"{"name": "Parus major", "count": 3}"#)?;
//! assert_eq!(value.str("name"), Some("Parus major"));
//! assert_eq!(value.number("count"), Some(3.0));
//! # Ok::<(), lite_json::JsonError>(())
//! ```

#![no_std]

extern crate alloc;

mod json;
mod json_error;

pub use json::Json;
pub use json_error::JsonError;
