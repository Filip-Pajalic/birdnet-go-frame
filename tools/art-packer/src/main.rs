//! Packs a fugleramme style folder's plates into one `.artb` bundle ([`art_pack::ArtBundle`]),
//! for firmware that draws its own page and carries its art in flash.
//!
//! ```text
//! art-packer <style folder> <out.artb> [--max-side PIXELS] [--species FILE] [--day DAYS]
//! ```
//!
//! Each plate is decoded, toned to the panel's paper and shrunk as the server does it, its
//! colours rounded to 32 levels a channel and its transparency to 16 (the six-ink dither cannot
//! tell), then stored run-length encoded and zlib-compressed: about 30 KB a species at the
//! default size. `--species` names a file with one scientific name a line, to
//! keep the bundle to the birds of one garden; without it every species is packed. `--day`
//! picks which plate of a species with several (days since 1970; 0 by default).
//!
//! The border's plants ([`art_webp::BorderSprays`]) go in too, by their names (`frost-top`
//! and so on), at the same size, so the firmware's pages wear the season's border.

use std::path::PathBuf;
use std::process::ExitCode;
use std::{env, fs};

use art_pack::{ArtBundle, ArtImage, ArtSource, BundleEntry, Rgba};
use art_webp::{ArtFolder, BorderSprays};

/// Plates are shrunk to this many pixels on their longest side by default: enough for the
/// largest bird on the half-size page an ESP32 draws for the 13.3" panel.
const MAX_SIDE: usize = 240;

/// What to pack, from the command line.
struct Options {
    folder: PathBuf,
    out: PathBuf,
    max_side: usize,
    species: Option<PathBuf>,
    day: u64,
}

fn main() -> ExitCode {
    let options: Options = match options(env::args().skip(1)) {
        Ok(options) => options,
        Err(problem) => {
            eprintln!("art-packer: {problem}");
            eprintln!(
                "usage: art-packer <style folder> <out.artb> [--max-side PIXELS] \
                 [--species FILE] [--day DAYS]"
            );
            return ExitCode::FAILURE;
        }
    };
    match pack(&options) {
        Ok((count, bytes)) => {
            println!(
                "art-packer: {count} species, {bytes} bytes, in {}",
                options.out.display()
            );
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("art-packer: {problem}");
            ExitCode::FAILURE
        }
    }
}

/// Packs the plates and writes the bundle. Returns how many species and bytes it holds.
fn pack(options: &Options) -> Result<(usize, usize), String> {
    let mut art: ArtFolder = ArtFolder::open(&options.folder)
        .map_err(|error| format!("{}: {error}", options.folder.display()))?
        .with_max_side(options.max_side)
        .with_day(options.day);
    let names: Vec<String> = match &options.species {
        Some(path) => fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .collect(),
        None => art.species(),
    };
    // Each species' name, its zlib-compressed picture and the picture's length unpacked.
    let mut packed: Vec<(String, Vec<u8>, usize)> = Vec::with_capacity(names.len());
    for name in names {
        let Some(picture) = art.art_for(&name) else {
            eprintln!("art-packer: no plate for {name:?}, left out");
            continue;
        };
        let artp: Vec<u8> = rounded(&picture).encode();
        packed.push((name, frame_http::zlib(&artp), artp.len()));
    }
    let species: usize = packed.len();
    let mut sprays: BorderSprays = BorderSprays::bundled().with_max_side(options.max_side);
    for name in BorderSprays::names() {
        if let Some(picture) = sprays.art_for(name) {
            let artp: Vec<u8> = rounded(&picture).encode();
            packed.push((name.to_owned(), frame_http::zlib(&artp), artp.len()));
        }
    }
    let bytes: Vec<u8> =
        ArtBundle::encode(packed.iter().map(|(name, zlib, unpacked_len)| BundleEntry {
            name,
            zlib,
            unpacked_len: *unpacked_len,
        }));
    fs::write(&options.out, &bytes)
        .map_err(|error| format!("{}: {error}", options.out.display()))?;
    Ok((species, bytes.len()))
}

/// Returns `picture` with its colours rounded to 32 levels and its transparency to 16, so runs
/// grow longer and compress better.
fn rounded(picture: &ArtImage) -> ArtImage {
    let level = |value: u8, steps: u16| -> u8 {
        let step: u16 = (u16::from(value) * steps + 127) / 255;
        u8::try_from(step * 255 / steps).unwrap_or(u8::MAX)
    };
    let pixels: Vec<Rgba> = picture
        .pixels()
        .iter()
        .map(|pixel| {
            Rgba::new(
                level(pixel.red, 31),
                level(pixel.green, 31),
                level(pixel.blue, 31),
                level(pixel.alpha, 15),
            )
        })
        .collect();
    ArtImage::new(picture.resolution(), pixels).unwrap_or_else(|| picture.clone())
}

/// Reads the command line.
fn options(mut args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut positional: Vec<String> = Vec::new();
    let mut max_side: usize = MAX_SIDE;
    let mut species: Option<PathBuf> = None;
    let mut day: u64 = 0;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--max-side" => max_side = number(args.next(), "--max-side")?,
            "--day" => day = number(args.next(), "--day")?,
            "--species" => {
                species = Some(PathBuf::from(args.next().ok_or("--species needs a file")?));
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ => positional.push(arg),
        }
    }
    let [folder, out] = <[String; 2]>::try_from(positional)
        .map_err(|_| String::from("give a style folder and an output file"))?;
    Ok(Options {
        folder: PathBuf::from(folder),
        out: PathBuf::from(out),
        max_side,
        species,
        day,
    })
}

fn number<T: std::str::FromStr>(value: Option<String>, flag: &str) -> Result<T, String> {
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("{flag} needs a number"))
}

#[cfg(test)]
mod test {
    use frame_core::Resolution;

    use super::*;

    #[test]
    fn rounding_keeps_the_ends_of_every_channel() {
        let pixels: Vec<Rgba> = vec![
            Rgba::new(0, 255, 3, 0),
            Rgba::new(255, 0, 130, 255),
            Rgba::new(9, 100, 250, 120),
        ];
        let picture = ArtImage::new(Resolution::new(3, 1), pixels).expect("three pixels");
        let out: Vec<Rgba> = rounded(&picture).pixels().to_vec();
        assert_eq!(out[0], Rgba::new(0, 255, 0, 0));
        assert_eq!(out[1].alpha, 255);
        assert_eq!(out[1].red, 255);
        assert!(out[2].alpha.abs_diff(120) <= 9);
    }
}
