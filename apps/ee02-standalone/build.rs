//! Links the firmware with esp-hal's linker scripts, and puts the art bundle where the firmware
//! includes it.
//!
//! `FRAME_ART` names an `.artb` file made by `tools/art-packer`; without it the bundle is empty
//! and the firmware lists the birds by name.

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    // esp-hal's scripts map IRAM read, write and execute on purpose.
    println!("cargo:rustc-link-arg=-Wl,--no-warn-rwx-segments");

    println!("cargo:rerun-if-env-changed=FRAME_ART");
    let Some(out) = env::var_os("OUT_DIR").map(PathBuf::from) else {
        panic!("cargo sets OUT_DIR for build scripts");
    };
    let bundle: Vec<u8> = match env::var_os("FRAME_ART").filter(|path| !path.is_empty()) {
        Some(path) => {
            let path = PathBuf::from(path);
            println!("cargo:rerun-if-changed={}", path.display());
            match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => panic!("FRAME_ART: {}: {error}", path.display()),
            }
        }
        None => Vec::new(),
    };
    if let Err(error) = fs::write(out.join("art.artb"), bundle) {
        panic!("cannot write the art bundle: {error}");
    }
}
