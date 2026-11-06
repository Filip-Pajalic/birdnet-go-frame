# birdnet-go-frame

A low-power e-ink bird frame for [BirdNET-Go](https://github.com/tphakala/birdnet-go). A small
Rust server reads your BirdNET-Go API and renders the birds heard in your garden. An ESP32 board
then shows the result on a 13.3" six-colour e-ink panel and runs on battery for months.

> **Status:** early development. Nothing here is ready to install yet.

This project is a Rust re-implementation of the server-and-panel idea from
[fugleramme](https://github.com/arnegiacomo/fugleramme) by Arne Giacomo Munthe-Kaas. It would not
exist without that project, its artwork curation and its ESP32 panel work. See
[Credits](#credits).

## How it works

```
BirdNET-Go (/api/v2) ──JSON──▶ apps/server  ──/frame.e6 or 304──▶ ESP32-S3 + e-ink panel
        (Unraid / any Docker host)                                 (wakes, redraws, sleeps)
```

1. **BirdNET-Go** listens to a microphone and identifies birds. You run it already. This project
   only reads its HTTP API.
2. **The server** (`apps/server`) is a single static binary (or a `FROM scratch` container). It polls
   BirdNET-Go, builds a collage of the species heard, dithers it to the panel's six inks, and
   serves it at `/frame.e6`. It re-renders only when the set of birds changes.
3. **The firmware** runs on a Seeed XIAO EE02 (ESP32-S3) board. It wakes on a timer and sends a
   conditional `GET /frame.e6`. It refreshes the panel only if the image changed, then goes back to
   deep sleep.

The `/frame.e6` format is byte-compatible with fugleramme's, so either side can be swapped for
the other.

## Hardware

| Part | Recommended |
| --- | --- |
| Board | [Seeed XIAO ePaper Display Board EE02](https://www.seeedstudio.com/XIAO-ePaper-Display-Board-ESP32-S3-EE02-p-6639.html) (ESP32-S3, PSRAM, LiPo charger) |
| Panel | [Seeed 13.3" Spectra 6, 1600x1200](https://www.seeedstudio.com/13-3inch-Six-Color-eInk-ePaper-Display-with-1200x1600-Pixels-p-6569.html) |
| Battery | 3.7 V single-cell LiPo, JST-PH 2.0, 2000-5000 mAh |
| Frame | IKEA RÖDALM 21x30 (A4) |
| Server | Anything that runs Docker or a Linux binary and can reach BirdNET-Go (e.g. Unraid) |

## Repository layout

Everything is a library crate. The crates under `apps/` are thin binaries that only choose which
libraries to wire together, so the same rendering code can run on a server, on a Raspberry Pi
driving the panel, or on the ESP32 itself.

| Path | What |
| --- | --- |
| `crates/frame-core` | `no_std`: E6 image format, palette, panel geometry, CRC |
| `crates/birdnet-model` | `no_std` + `alloc`: detection and species types, JSON parsing |
| `crates/birdnet-client` | `no_std` + `alloc`: BirdNET-Go API client, generic over an HTTP transport |
| `crates/transport-ureq`, `crates/transport-esp` | HTTP transports for Linux and the ESP32 |
| `crates/art-pack` | `no_std` + `alloc`: compact pre-decoded artwork format and reader |
| `crates/frame-layout` | `no_std` + `alloc`: picks art, sizes birds by body mass, packs the page |
| `crates/frame-raster` | `no_std` + `alloc`: draws art and text, dithers to six inks |
| `crates/panel-t133a01` | `no_std`: 13.3" Spectra 6 driver on `embedded-hal` 1.0 traits |
| `crates/frame-http` | Serves `/frame.e6`, `/frame.png`, `/preview.png` |
| `apps/server` | The server for Unraid / Docker |
| `apps/ee02-thin` | ESP32 firmware that fetches the finished image (best battery life) |
| `apps/ee02-standalone` | ESP32 firmware that talks to BirdNET-Go and renders on the device |
| `apps/linux-panel` | Renders and drives the panel directly from a Raspberry Pi |
| `tools/art-packer` | Converts the webp artwork into `art-pack` files once |
| `assets/` | Fonts and small data files, each with its own `ATTRIBUTION.md` |
| `deploy/` | Dockerfile and Unraid template |

## Configuration (planned)

The server is configured with environment variables:

| Variable | Default | Meaning |
| --- | --- | --- |
| `BIRDNET_URL` | - | BirdNET-Go base URL, e.g. `http://192.168.1.10:8080` |
| `PANEL` | `13.3` | Panel size |
| `ROTATION` | `0` | `0`, `90`, `180` or `270` |
| `POLL_MINUTES` | `15` | How often the panel should check for a new page |
| `WINDOW_HOURS` | `24` | How far back detections count |
| `LOCALE` | `en` | Language for species names |
| `QUIET_HOURS` | - | e.g. `22-06`; the panel sleeps longer during these hours |
| `ART_DIR` | `/data/art` | Where `scripts/fetch-art.sh` put the artwork |

## Development

```bash
cargo build -p server --release         # server, stable Rust
cargo +nightly fmt --check              # formatting (Google style options need nightly rustfmt)
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

The ESP32 apps use Espressif's Rust toolchain (`espup install`) and build from their `apps/` folder.

### Code style

All Rust code follows [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
Formatting and the lints it names are enforced in CI (see `rustfmt.toml` and the workspace
`[lints]` table). The rest is covered by the review checklist in [`STYLE.md`](STYLE.md).

## Credits

- **[fugleramme](https://github.com/arnegiacomo/fugleramme)** by Arne Giacomo Munthe-Kaas: the
  concept, the `/frame.e6` format, the EE02 panel driving sequence and the curated 1800s bird
  artwork. Code under the MIT License; artwork under CC BY-SA 4.0.
- **[BirdNET-Go](https://github.com/tphakala/birdnet-go)** by tphakala and contributors:
  the bird detection this frame displays. The BirdNET model is by the Cornell Lab of Ornithology
  and Chemnitz University of Technology. Taxonomy data is powered by eBird.org.
- **[Seeed_GFX2](https://github.com/Seeed-Studio/Seeed_GFX2)** (MIT): reference driver for the
  13.3" T133A01 panel.
- **AVONET** (Tobias et al. 2022, *Ecology Letters*,
  [doi:10.1111/ele.13898](https://doi.org/10.1111/ele.13898)): bird body masses, CC BY 4.0.
- **[OpenFauna](https://github.com/tphakala/openfauna)**: scientific-name aliases, CC BY-SA 4.0.

Full notices are in [`NOTICE.md`](NOTICE.md).

This project is not affiliated with or endorsed by BirdNET-Go, the Cornell Lab of Ornithology,
Seeed Studio, or fugleramme.

## License

- **Code:** MIT. See [`LICENSE`](LICENSE). Portions ported from fugleramme keep their original
  MIT copyright notice (see [`NOTICE.md`](NOTICE.md)).
- **Bird artwork:** not included in this repository. `scripts/fetch-art.sh` downloads
  fugleramme's art styles together with their `ATTRIBUTION.md` and `manifest.json`. The
  `classic` style is **CC BY-SA 4.0**. If you share pictures of your frame's output, credit the
  artwork and share alike.
- **Fonts** in `assets/fonts/`: SIL Open Font License 1.1, with each font's licence alongside.
- **Data files** in `assets/`: under their own licences, listed in `assets/ATTRIBUTION.md`.
- **BirdNET-Go** is a separate program under CC BY-NC-SA 4.0 (non-commercial). This project
  only talks to its HTTP API and contains none of its code.
