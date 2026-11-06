# Supported hardware

Which panels the server lays pages out for, which boards the firmware runs on, and which
combinations work today. "Builds" means CI compiles it on every push; nothing on the panel side
has been tried on real hardware yet, so please report what you find.

## Combinations

| Board | Panel | Firmware | State |
| --- | --- | --- | --- |
| Seeed XIAO ePaper Display Board **EE02** (ESP32-S3, 8 MB octal PSRAM) | Seeed **13.3" Spectra 6**, 1600x1200 (two T133A01 controllers) | `apps/ee02-thin`, with the server | **Supported**: builds, logic tested on the host, untried on hardware. [Flash it](../how-to/flash-ee02-thin.md). |
| Seeed **reTerminal E1002** (ESP32-S3, 8 MB octal PSRAM, 32 MB flash) | Its **7.3" Spectra 6**, 800x480 (ED2208) | `apps/ee02-thin --features board-e1002`, with the server | **Builds**, untried on hardware; same guide, with the feature |
| EE02 or reTerminal E1002 | 13.3" or 7.3" | `apps/ee02-standalone`, **no server**: draws its own page | **Builds**, drawing tested on the host, untried on hardware. [Flash it](../how-to/flash-ee02-standalone.md). |
| Raspberry Pi (any with a 40-pin header) | 13.3" or 7.3" Spectra 6 on an SPI driver board | `apps/linux-panel`, **no server** | **Builds**, released for 64-bit Pi OS, tested against a mock SPI bus, untried on a panel. [Run it](../how-to/run-on-raspberry-pi.md). |
| EE02 | 13.3" Spectra 6 | fugleramme's MicroPython firmware, a few lines changed | Works with the server's `/panel`; [how](../how-to/show-on-ee02.md) |
| Any other board with a 7.3" Spectra 6 panel | 7.3" | Your own, reading `/panel?panel=7.3` | The server lays out and serves the page; `panel-ed2208` drives the panel on any `embedded-hal` 1.0 HAL |

## Panels

| Panel | Size on `/panel` | Server | Driver crate | Firmware and apps |
| --- | --- | --- | --- | --- |
| 13.3" Spectra 6, 1600x1200 | 960,000 bytes | Yes (`PANEL=13.3`, the default) | `panel-t133a01` | `ee02-thin`, `ee02-standalone`, `linux-panel` |
| 7.3" Spectra 6, 800x480 (reTerminal E1002, Waveshare 7.3" (E)) | 192,000 bytes | Yes (`PANEL=7.3` or `?panel=7.3`) | `panel-ed2208` | `ee02-thin` and `ee02-standalone` with `board-e1002`, `linux-panel` |

Both drivers are tested against a mock SPI bus: the command sequence, the bytes and the BUSY
handling. The ED2208's register values follow Zephyr's driver for the same panel.

Both are six-ink panels (black, white, yellow, red, blue, green); the byte layouts are in
[The panel format](panel-format.md). For anything else, such as a browser, a tablet or a picture
frame that shows images, the server renders a PNG at any size: see
[HTTP endpoints](http-endpoints.md#sizes).

## ESP32 models

The thin firmware needs room for the whole page in RAM, because it inflates the page before
sending it to the panel; the standalone firmware also holds the drawing, the font and each
bird's picture. For the 13.3" panel that means PSRAM.

| Chip | Rust target | 13.3" (960 KB page) | 7.3" (192 KB page) | Notes |
| --- | --- | --- | --- | --- |
| **ESP32-S3** with PSRAM (EE02, reTerminal E1002, XIAO ESP32-S3 Sense or Plus, ESP32-S3-WROOM-1 N*R2/R8) | `xtensa-esp32s3-none-elf` | **Yes**: both firmwares target it | **Yes**: `board-e1002` | Other S3 boards need a module in `crates/glass-esp` with their pins and a `board.rs` entry, and for quad PSRAM `PsramMode::QuadSpi` in `main.rs`. Standalone needs 8 MB of PSRAM for the 13.3" page |
| ESP32-S3 without PSRAM | `xtensa-esp32s3-none-elf` | No: the page does not fit in 512 KB of SRAM next to Wi-Fi | Thin firmware possibly; standalone no | |
| ESP32 (original) with PSRAM (WROVER) | `xtensa-esp32-none-elf` | Possible, untried | Possible | Needs a build of the firmware for that chip |
| ESP32-C3, C6 | `riscv32imc` / `riscv32imac` | No: no PSRAM | Thin firmware possible, tight | Would also need streaming inflate |
| ESP32-C5, C61 with PSRAM | RISC-V | Possible, untried | Possible | esp-hal support for their radios is newer |
| ESP32-H2, P4 | RISC-V | No: no Wi-Fi of their own | No | |

The libraries the firmware shares with the server (`frame-core`, `panel-wake`, `panel-setup`,
`panel-t133a01`, `panel-ed2208`, and for the standalone firmware `frame-artist`,
`frame-raster`, `frame-layout`, `art-pack` and `frame-standalone`) are `no_std` and build for
any of these; only the two firmware apps, `crates/glass-esp` and `crates/transport-esp` are
tied to the ESP32-S3 today.

## Adding a combination

- **Another ESP32-S3 board:** add a module to `crates/glass-esp` with its panel pins and power,
  a feature and entries in both firmwares' `board.rs`, the PSRAM mode if it has quad PSRAM, and
  document the board here.
- **Another Spectra 6 panel:** write a driver crate next to `panel-t133a01` and `panel-ed2208`
  against `embedded-hal` 1.0, implement `frame_core::Glass` for it, and add its `PanelSize`.
- **Another chip:** add its feature to the esp crates in a new app under `apps/`, keeping the
  logic in the shared libraries.
