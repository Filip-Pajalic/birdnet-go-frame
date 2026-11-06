# Supported hardware

Which panels the server lays pages out for, which boards the firmware runs on, and which
combinations work today. "Builds" means CI compiles it on every push; nothing on the panel side
has been tried on real hardware yet, so please report what you find.

## Combinations

| Board | Panel | Firmware | State |
| --- | --- | --- | --- |
| Seeed XIAO ePaper Display Board **EE02** (ESP32-S3, 8 MB octal PSRAM) | Seeed **13.3" Spectra 6**, 1600x1200 (two T133A01 controllers) | `apps/ee02-thin` | **Supported**: builds, logic tested on the host, untried on hardware. [Flash it](../how-to/flash-ee02-thin.md). |
| EE02 | 13.3" Spectra 6 | fugleramme's MicroPython firmware, a few lines changed | Works with the server's `/panel`; [how](../how-to/show-on-ee02.md) |
| Any board with a 7.3" Spectra 6 panel, 800x480 | 7.3" | Your own, reading `/panel?panel=7.3` | **Server only**: the server lays out and serves the page; no firmware or driver in this repository yet |
| Raspberry Pi | 13.3" Spectra 6 | `apps/linux-panel` | Planned |
| ESP32-S3 drawing its own page, no server | 13.3" Spectra 6 | `apps/ee02-standalone` | Planned |

## Panels

| Panel | Size on `/panel` | Server | Driver crate | Firmware |
| --- | --- | --- | --- | --- |
| 13.3" Spectra 6, 1600x1200 | 960,000 bytes | Yes (`PANEL=13.3`, the default) | `panel-t133a01` | `apps/ee02-thin` |
| 7.3" Spectra 6, 800x480 | 192,000 bytes | Yes (`PANEL=7.3` or `?panel=7.3`) | None yet | None yet |

Both are six-ink panels (black, white, yellow, red, blue, green); the byte layouts are in
[The panel format](panel-format.md). For anything else, such as a browser, a tablet or a picture
frame that shows images, the server renders a PNG at any size: see
[HTTP endpoints](http-endpoints.md#sizes).

## ESP32 models

The firmware needs room for the whole page in RAM, because it inflates the page before sending it
to the panel. For the 13.3" panel that means PSRAM.

| Chip | Rust target | 13.3" (960 KB page) | 7.3" (192 KB page) | Notes |
| --- | --- | --- | --- | --- |
| **ESP32-S3** with PSRAM (EE02, XIAO ESP32-S3 Sense or Plus, ESP32-S3-WROOM-1 N*R2/R8) | `xtensa-esp32s3-none-elf` | **Yes**: `ee02-thin` targets it | Fits; needs a 7.3" driver | Other S3 boards need the pin table in `apps/ee02-thin/src/glass.rs` and, for quad PSRAM, `PsramMode::QuadSpi` in `main.rs` |
| ESP32-S3 without PSRAM | `xtensa-esp32s3-none-elf` | No: the page does not fit in 512 KB of SRAM next to Wi-Fi | Possibly | |
| ESP32 (original) with PSRAM (WROVER) | `xtensa-esp32-none-elf` | Possible, untried | Possible | Needs a build of `ee02-thin` for that chip |
| ESP32-C3, C6 | `riscv32imc` / `riscv32imac` | No: no PSRAM | Possible, tight | Would also need streaming inflate |
| ESP32-C5, C61 with PSRAM | RISC-V | Possible, untried | Possible | esp-hal support for their radios is newer |
| ESP32-H2, P4 | RISC-V | No: no Wi-Fi of their own | No | |

The libraries the firmware shares with the server (`frame-core`, `panel-wake`,
`panel-t133a01`) are `no_std` and build for any of these; only `apps/ee02-thin` and
`crates/transport-esp` are tied to the ESP32-S3 today.

## Adding a combination

- **Another ESP32-S3 board, same panel:** change the pins in `glass.rs`, the PSRAM mode if it has
  quad PSRAM, and document the board here.
- **The 7.3" panel:** write a driver crate next to `panel-t133a01` against `embedded-hal` 1.0,
  and an app that asks for `PanelSize::Inches7Point3`. `panel-wake` already handles the size.
- **Another chip:** add its feature to the esp crates in a new app under `apps/`, keeping the
  logic in the shared libraries.
