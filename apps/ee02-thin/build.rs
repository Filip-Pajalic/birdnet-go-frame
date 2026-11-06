//! Links the firmware with esp-hal's linker scripts.
//!
//! Link arguments live here rather than in `.cargo/config.toml`, so a `RUSTFLAGS` set by CI or by
//! hand cannot drop them.

fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    println!("cargo:rustc-link-arg=-Tlinkall.x");
    // esp-hal's scripts map IRAM read, write and execute on purpose.
    println!("cargo:rustc-link-arg=-Wl,--no-warn-rwx-segments");
}
