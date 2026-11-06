# birdnet-go-frame

A low-power e-ink bird frame for [BirdNET-Go](https://github.com/tphakala/birdnet-go). A small
Rust server reads your BirdNET-Go API and renders the birds heard in your garden. An ESP32 board
then shows the result on a 13.3" six-colour e-ink panel and runs on battery for months.

> **Status:** early development. The server runs and serves the illustrated collage (or a text
> list) that the panel can show. Until real art is supported the birds are generated pictures,
> so it works out of the box. The Rust ESP32 firmware comes next; until then, fugleramme's
> MicroPython EE02 firmware can show this server's `/frame.e6`.

![The collage, drawn from the built-in simulated garden](docs/images/collage-mock.png)

*The page as it hangs, from `BIRDNET_URL=mock`. Heavier birds are drawn bigger; the panel shows
it dithered to its six inks.*

## Quick start (no BirdNET-Go needed)

```bash
BIRDNET_URL=mock cargo run --release -p server
# open http://localhost:8080/
```

`BIRDNET_URL=mock` uses the built-in simulated station: a garden of 18 European species with a
dawn chorus, deterministic for a given time. To test over real HTTP instead, run the fake
BirdNET-Go and point the server at it:

```bash
cargo run --release -p fake-birdnet -- 127.0.0.1:8090
BIRDNET_URL=http://127.0.0.1:8090 LISTEN=127.0.0.1:8081 cargo run --release -p server
```

The fake speaks the same `/api/v2` paths as BirdNET-Go, so fugleramme can use it too.

With your own BirdNET-Go:

```bash
BIRDNET_URL=http://192.168.1.10:8080 UTC_OFFSET_MINUTES=60 cargo run --release -p server
```

| Endpoint | What |
| --- | --- |
| `/` | A page showing the preview |
| `/frame.e6` | The page for the 13.3" panel (E6F1, ETag, 304 when unchanged) |
| `/frame.png` | Six-colour PNG in the panel's native orientation |
| `/preview.png` | Full-colour PNG as it hangs |
| `/healthz` | `ok` once a page exists |

This project is a Rust re-implementation of the server-and-panel idea from
[fugleramme](https://github.com/arnegiacomo/fugleramme) by Arne Giacomo Munthe-Kaas. It would not
exist without that project, its artwork curation and its ESP32 panel work. See
[Credits](#credits).

## Documentation

Full docs, organised by [Diátaxis](https://diataxis.fr/), are in [`docs/`](docs/README.md): a
tutorial, how-to guides, reference, and explanations of the architecture and of how the image is
made (with diagrams).

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

| Path | What | State |
| --- | --- | --- |
| `crates/frame-core` | `no_std`: E6 image format, inks, panel geometry, CRC-32 | Done; byte-compatible with fugleramme |
| `crates/birdnet-model` | `no_std` + `alloc`: BirdNET-Go response types and a small JSON reader | Done |
| `crates/birdnet-client` | `no_std` + `alloc`: API client, generic over an `HttpTransport` | Done (no private-mode login yet) |
| `crates/birdnet-mock` | `no_std` + `alloc`: simulated BirdNET-Go and an in-process transport | Done |
| `crates/lan-http` | Plain HTTP/1.1 GET client and small server (std only) | Done |
| `crates/transport-lan` | `HttpTransport` for Linux, macOS and Windows | Done |
| `crates/art-pack` | `no_std` + `alloc`: bird cut-outs, the `.artp` format, the `ArtSource` trait | Done |
| `crates/art-mock` | `no_std` + `alloc`: generated bird pictures for any species | Done |
| `crates/bird-sizes` | `no_std`: AVONET body masses for the 6,421 species BirdNET detects | Done |
| `crates/frame-layout` | `no_std` + `alloc`: what goes where on the page | Collage and text list done |
| `crates/frame-raster` | `no_std` + `alloc`: draws pictures and text, dithers to six inks | Done |
| `crates/frame-http` | Serves `/frame.e6`, `/frame.png`, `/preview.png`; writes PNGs | Done |
| `apps/server` | The server for Unraid / Docker | Done |
| `apps/fake-birdnet` | The simulated BirdNET-Go over HTTP | Done |
| `crates/transport-esp`, `crates/panel-t133a01`, `tools/art-packer` | ESP32 networking, panel driver, real art | Planned |
| `apps/ee02-thin`, `apps/ee02-standalone`, `apps/linux-panel` | ESP32 and Pi firmware | Planned |
| `assets/` | Fonts, each with its licence and an `ATTRIBUTION.md` | |
| `deploy/` | Dockerfile | |
| `.github/` | CI, release and container workflows, Dependabot, issue and PR templates | |

## Configuration

The server is configured with environment variables:

| Variable | Default | Meaning |
| --- | --- | --- |
| `BIRDNET_URL` | required | BirdNET-Go base URL, e.g. `http://192.168.1.10:8080`, or `mock` |
| `LISTEN` | `0.0.0.0:8080` | Address to serve on |
| `ROTATION` | `0` | `0`, `90`, `180` or `270`, counter-clockwise from landscape |
| `POLL_MINUTES` | `15` | How long the panel sleeps between checks (sent in the page header) |
| `WINDOW_HOURS` | `24` | How far back detections count |
| `UTC_OFFSET_MINUTES` | `0` | Your time zone, for the date range and the "Updated" time |
| `REFRESH_SECONDS` | `60` | How often the server asks BirdNET-Go for news |
| `HEADING` | `Heard in the garden` | Page heading |
| `LAYOUT` | `collage` | `collage` (pictures, sized by body mass) or `list` (names and counts) |
| `MAX_BIRDS` | `15` | Most birds pictured; the rest are counted in one line |

The page is redrawn only when the set of species changes, so most panel wakes get a 304.

## Install

**Docker / Unraid.** Images for amd64 and arm64 are published to GitHub Container Registry:

```bash
docker run -p 8080:8080 -e BIRDNET_URL=http://192.168.1.10:8080 ghcr.io/filip-pajalic/birdnet-go-frame:edge
```

`:edge` follows `main`; version tags (`:0.2.0`, `:0.2`) appear with releases. The image is
`FROM scratch` with one static binary of about 1.5 MB and runs as an unprivileged user. To build
it yourself: `docker build -f deploy/Dockerfile -t birdnet-go-frame .`

**Binaries.** Each [release](https://github.com/Filip-Pajalic/birdnet-go-frame/releases) has
`birdnet-go-frame` and `fake-birdnet` for Linux (x86_64 and aarch64, static), macOS (Intel and
Apple silicon) and Windows, with `SHA256SUMS.txt`.

## Dependencies

The server uses **eight third-party crates**, all of them the font rasteriser `fontdue` and its
own dependencies. JSON parsing, HTTP and PNG encoding are small, tested modules in this
repository. Why, and the rules for adding one: [docs/explanation/dependencies.md](docs/explanation/dependencies.md).
`cargo deny` checks licences, advisories and duplicates in CI.

## Development

```bash
cargo build -p server --release         # server, stable Rust 1.87 or newer
cargo run -p server                     # with BIRDNET_URL=mock for a demo
cargo +nightly fmt --check              # formatting (Google style options need nightly rustfmt)
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Contributions are welcome, from forks too: see [CONTRIBUTING.md](CONTRIBUTING.md), the
[code of conduct](CODE_OF_CONDUCT.md) and [SECURITY.md](SECURITY.md). CI runs on every pull
request without secrets: formatting, clippy, tests on Linux, macOS and Windows, the minimum Rust
version, a bare-metal build of the `no_std` crates, docs and `cargo deny`. Pushing a `v*` tag
builds the release binaries and the container image.

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
  [doi:10.1111/ele.13898](https://doi.org/10.1111/ele.13898)): bird body masses, CC BY 4.0,
  via fugleramme's table (`crates/bird-sizes/data/`). The size exponent (0.14) is fugleramme's.
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
- **Bird sizes** in `crates/bird-sizes/data/bird_sizes.csv`: AVONET, CC BY 4.0; see the
  `ATTRIBUTION.md` beside it. The `bird-sizes` crate is therefore `MIT AND CC-BY-4.0`.
- **BirdNET-Go** is a separate program under CC BY-NC-SA 4.0 (non-commercial). This project
  only talks to its HTTP API and contains none of its code.
