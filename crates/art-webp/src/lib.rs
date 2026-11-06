//! Bird art from a fugleramme-style folder, as an [`art_pack::ArtSource`].
//!
//! A style folder holds `birds/<genus>-<species>[-N].webp` cut-outs and an optional
//! `geometry.json` with the box around the bird on each plate, as
//! [fugleramme](https://github.com/arnegiacomo/fugleramme) ships them. [`ArtFolder`] picks one
//! plate per species per day, decodes it, shrinks it, and tones its paper halo to the page's
//! paper so every plate looks printed on one sheet. The artwork itself is not part of this
//! repository; see `scripts/fetch-art.sh`.

mod art_folder;
mod geometry;
mod halo;
mod plate;

pub use art_folder::ArtFolder;
