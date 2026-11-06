//! The words printed on the page, in the viewer's language.
//!
//! [`Translations`] reads a simple JSON file, one object per language code, each mapping a
//! key to its text; a bundled file covers a dozen European languages. [`PageText`] holds one
//! language's words, falling back key by key to English. Bird names are not here: BirdNET-Go
//! already sends them in its own configured language. `no_std` with `alloc`.
//!
//! ```
//! use page_text::Translations;
//!
//! let words = Translations::bundled().page_text("sv-SE");
//! assert_eq!(words.nothing_heard, "Inga fåglar hörda än");
//! assert_eq!(words.updated("07:30"), "Uppdaterad 07:30");
//! ```

#![no_std]

extern crate alloc;

mod page_text;
mod translations;

pub use page_text::PageText;
pub use translations::Translations;
