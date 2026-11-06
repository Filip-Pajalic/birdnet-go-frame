# birdnet-go-frame

A low-power e-ink bird frame for [BirdNET-Go](https://github.com/tphakala/birdnet-go). A small
Rust server reads your BirdNET-Go API and renders the birds heard in your garden. An ESP32 board
then shows the result on a 13.3" six-colour e-ink panel and runs on battery for months.

> **Status:** early development. The server is done: it serves the illustrated collage (or a
> text list), drawn with fugleramme's 1800s plates when you fetch them or with generated birds
> out of the box, logs in to a private BirdNET-Go, and installs on Unraid from a template. On the
> panel side, the driver for the 13.3" panel and the logic of each wake are written and tested
> against mocks and the real server; the ESP32 app that ties them to Wi-Fi and the board comes
> next and needs the hardware to prove. Until then, fugleramme's MicroPython EE02 firmware needs a
> few lines changed to read `/panel` (see [Show the page on an EE02](docs/how-to/show-on-ee02.md)).

![The collage with fugleramme's classic plates](docs/images/collage-classic.jpg)

*The page as it hangs, from the simulated garden with `ART_DIR` set to fugleramme's `classic`
style. Birds are packed by their outlines outwards from the middle, heavier birds bigger, as
fugleramme does, and sized as big as the page allows so the cluster reaches its corners. Artwork: 1800s plates curated and edited by
[fugleramme](https://github.com/arnegiacomo/fugleramme), CC BY-SA 4.0 (sources in each style's
`ATTRIBUTION.md`).*

## Quick start (no BirdNET-Go needed)

```bash
BIRDNET_URL=mock cargo run --release -p server
# open http://localhost:8080/
```

`BIRDNET_URL=mock` uses the built-in simulated station: a garden of 35 European species with a
dawn chorus, deterministic for a given time. Without artwork the birds are generated pictures
([example](docs/images/collage-mock.png)). For the real plates, fetch fugleramme's art once
(about 250 MB, never committed here) and point `ART_DIR` at it:

```bash
scripts/fetch-art.sh                     # downloads art/classic
BIRDNET_URL=mock ART_DIR=art/classic LANGUAGE=sv cargo run --release -p server
``` To test over real HTTP instead, run the fake
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
| `/panel` | The page as the panel's controllers take it, zlib-compressed (about 215 KB instead of 960 KB), ETag, 304 when unchanged |
| `/frame.png` | Six-colour PNG in the panel's native orientation |
| `/preview.png` | Full-colour PNG as it hangs |
| `/healthz` | `ok` once a page exists |

Each client can ask for its own size: `?panel=7.3` or `?panel=13.3` with `&rotation=90` for
e-ink panels, or `?w=1920&h=1080` for any screen (PNG only). Each size is drawn once per change
and cached; see [HTTP endpoints](docs/reference/http-endpoints.md#sizes).

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
BirdNET-Go (/api/v2) ──JSON──▶ apps/server  ───/panel or 304───▶ ESP32-S3 + e-ink panel
        (Unraid / any Docker host)                                 (wakes, redraws, sleeps)
```

1. **BirdNET-Go** listens to a microphone and identifies birds. You run it already. This project
   only reads its HTTP API.
2. **The server** (`apps/server`) is a single static binary (or a `FROM scratch` container). It polls
   BirdNET-Go, builds a collage of the species heard, dithers it to the panel's six inks, and
   serves it at `/panel`. It re-renders only when the set of birds changes.
3. **The firmware** runs on a Seeed XIAO EE02 (ESP32-S3) board. It wakes on a timer and sends a
   conditional `GET /panel`. It refreshes the panel only if the image changed, then goes back to
   deep sleep.

`/panel` sends the same pixel bytes as fugleramme's `/frame.e6`, without its header and
compressed, so the radio moves about a quarter of the bytes per new page; see
[The panel format](docs/reference/panel-format.md).

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
| `crates/frame-core` | `no_std`: inks, panel geometry, the controllers' byte layout, CRC-32, a zlib inflater | Done; pixel bytes match fugleramme's |
| `crates/lite-json` | `no_std` + `alloc`: a small JSON reader, in place of serde | Done |
| `crates/birdnet-model` | `no_std` + `alloc`: BirdNET-Go response types | Done |
| `crates/birdnet-client` | `no_std` + `alloc`: API client, generic over an `HttpTransport`, with BirdNET-Go's password login | Done |
| `crates/birdnet-mock` | `no_std` + `alloc`: simulated BirdNET-Go (private mode and login included) and an in-process transport | Done |
| `crates/lan-http` | Plain HTTP/1.1 client and small server (std only) | Done |
| `crates/transport-lan` | `HttpTransport` for Linux, macOS and Windows | Done |
| `crates/art-pack` | `no_std` + `alloc`: bird cut-outs, the `.artp` format, the `ArtSource` trait | Done |
| `crates/art-mock` | `no_std` + `alloc`: generated bird pictures for any species | Done |
| `crates/art-webp` | fugleramme's art folders: a plate per species per day, paper halo toned and feathered | Done |
| `crates/page-text` | `no_std` + `alloc`: the page's words in a dozen languages, from a translations JSON | Done |
| `crates/bird-sizes` | `no_std`: AVONET body masses for the 6,421 species BirdNET detects | Done |
| `crates/frame-layout` | `no_std` + `alloc`: what goes where; the collage's spiral packing by outline, out to the page's corners | Collage and text list done |
| `crates/frame-raster` | `no_std` + `alloc`: draws pictures and text, boosts and dithers to six inks | Done |
| `crates/frame-http` | Serves `/panel`, `/frame.png`, `/preview.png`; a small zlib compressor for them | Done |
| `apps/server` | The server for Unraid / Docker | Done |
| `apps/fake-birdnet` | The simulated BirdNET-Go over HTTP | Done |
| `crates/panel-t133a01` | `no_std`: the 13.3" panel's two controllers over `embedded-hal` 1.0, tested against a mock bus | Done; untried on hardware |
| `crates/panel-wake` | `no_std` + `alloc`: each wake's request, unpacking and checking the page, sleep and back-off, RTC state | Done; tested against the real server |
| `crates/transport-esp`, `tools/art-packer` | ESP32 networking, art on an SD card | Planned |
| `apps/ee02-thin` | ESP32 firmware: Wi-Fi, `panel-wake` and `panel-t133a01` on the EE02 | Next; needs the board |
| `apps/ee02-standalone`, `apps/linux-panel` | ESP32 that draws its own page; a Raspberry Pi panel | Planned |
| `assets/` | Fonts, each with its licence and an `ATTRIBUTION.md` | |
| `deploy/` | Dockerfile, Unraid template | |
| `scripts/fetch-art.sh` | Downloads a fugleramme art style for `ART_DIR` | |
| `.github/` | CI, release and container workflows, Dependabot, issue and PR templates | |

## Configuration

The server is configured with environment variables:

| Variable | Default | Meaning |
| --- | --- | --- |
| `BIRDNET_URL` | required | BirdNET-Go base URL, e.g. `http://192.168.1.10:8080`, or `mock` |
| `BIRDNET_PASSWORD` | unset | BirdNET-Go's password, only when its private mode is on |
| `LISTEN` | `0.0.0.0:8080` | Address to serve on |
| `PANEL` | `13.3` | Panel for clients that do not name one: `13.3` or `7.3` |
| `ROTATION` | `0` | `0`, `90`, `180` or `270`, counter-clockwise from landscape |
| `POLL_MINUTES` | `15` | How long the panel sleeps between checks (sent as `Cache-Control: max-age` on `/panel`) |
| `QUIET_HOURS` | unset | Local hours the panel sleeps through in one go, e.g. `22-6` |
| `WINDOW_HOURS` | `24` | How far back detections count |
| `UTC_OFFSET_MINUTES` | `0` | Your time zone, for the date range and the "Updated" time |
| `REFRESH_SECONDS` | `60` | How often the server asks BirdNET-Go for news |
| `ART_DIR` | unset | fugleramme art style folder, e.g. `art/classic`; unset draws generated birds |
| `LAYOUT` | `collage` | `collage` (pictures packed by outline, sized by body mass) or `list` (names and counts) |
| `MAX_BIRDS` | `0` | Most birds on a page; `0` for every bird heard |
| `NAMES` | `true` | Whether the collage writes each bird's name under it |
| `HEADING` | unset | A heading over the page; the collage has none by default, like fugleramme |
| `LANGUAGE` | `en` | Language of the page's own words, e.g. `sv`, `nb`, `de` (bird names come from BirdNET-Go) |
| `TRANSLATIONS` | unset | A translations JSON adding languages or overriding words |

The page is redrawn only when the set of species changes (or a new day picks other plates), so
most panel wakes get a 304. Details, and the translations format, are in
[docs/reference/configuration.md](docs/reference/configuration.md).

## Install

**Unraid.** Install from the template in [`deploy/unraid`](deploy/unraid/birdnet-go-frame.xml); the
[Docker and Unraid how-to](docs/how-to/run-in-docker-or-unraid.md) walks through it, plates included.

**Docker.** Images for amd64 and arm64 are published to GitHub Container Registry:

```bash
docker run -p 8080:8080 -e BIRDNET_URL=http://192.168.1.10:8080 \
  -v /mnt/user/appdata/birdnet-go-frame/art:/art:ro -e ART_DIR=/art/classic \
  ghcr.io/filip-pajalic/birdnet-go-frame:edge
```

Fill the art folder once with `scripts/fetch-art.sh classic /mnt/user/appdata/birdnet-go-frame/art`
(or leave out the two art lines for generated birds).

`:edge` follows `main`; version tags (`:0.2.0`, `:0.2`) appear with releases. The image is
`FROM scratch` with one static binary of about 1.7 MB and runs as an unprivileged user. To build
it yourself: `docker build -f deploy/Dockerfile -t birdnet-go-frame .`

**Binaries.** Each [release](https://github.com/Filip-Pajalic/birdnet-go-frame/releases) has
`birdnet-go-frame` and `fake-birdnet` for Linux (x86_64 and aarch64, static), macOS (Intel and
Apple silicon) and Windows, with `SHA256SUMS.txt`.

## Dependencies

The server uses **eleven third-party crates**: the font rasteriser `fontdue` with its own
dependencies, and the webp decoder `image-webp` with its two. The panel side adds one,
`embedded-hal`, with no dependencies of its own. JSON parsing, HTTP, PNG and zlib are small,
tested modules in this repository. Why, and the rules for adding one: [docs/explanation/dependencies.md](docs/explanation/dependencies.md).
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

The ESP32 apps will use Espressif's Rust toolchain (`espup install`) and build from their `apps/`
folder. The panel crates they build on (`panel-t133a01`, `panel-wake`) already build for bare
metal in CI and test on any machine.

### Code style

All Rust code follows [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
Formatting and the lints it names are enforced in CI (see `rustfmt.toml` and the workspace
`[lints]` table). The rest is covered by the review checklist in [`STYLE.md`](STYLE.md).

## Credits

- **[fugleramme](https://github.com/arnegiacomo/fugleramme)** by Arne Giacomo Munthe-Kaas: the
  concept, the controllers' pixel layout, the EE02 panel driving sequence, the collage (spiral packing
  by outline, names tucked under each bird, mass-based sizing), the paper-halo blending and
  panel boost, and the curated 1800s bird artwork this server draws. Code under the MIT License;
  artwork under CC BY-SA 4.0. Credit appears here and in `NOTICE.md`, never on the page itself.
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
