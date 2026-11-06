# birdnet-go-frame

A low-power e-ink bird frame for [BirdNET-Go](https://github.com/tphakala/birdnet-go). A small
Rust server reads your BirdNET-Go API and renders the birds heard in your garden. An ESP32 board
then shows the result on a 13.3" six-colour e-ink panel and runs on battery for months.

> **Status:** early development. The server is done: it serves the illustrated collage (or a
> text list), drawn with fugleramme's 1800s plates (fetched once with `scripts/fetch-art.sh`; until then
> the birds are listed by name), logs in to a private BirdNET-Go, and installs on Unraid from a template. The
> EE02 firmware (`apps/ee02-thin`) builds for the ESP32-S3 in CI and its logic is tested on any
> machine, but it has not run on the hardware yet: see
> [Build and flash the EE02 firmware](docs/how-to/flash-ee02-thin.md), and please report what you
> find. fugleramme's MicroPython firmware also works with a few lines changed
> ([how](docs/how-to/show-on-ee02.md)).

![The collage with fugleramme's classic plates](docs/images/collage-classic.jpg)

*The page as it hangs, from the simulated garden with `ART_DIR` set to fugleramme's `classic`
style. Birds are packed by their outlines outwards from the middle, heavier birds bigger, as
fugleramme does, and sized as big as the page allows so the cluster reaches its corners. Artwork: 1800s plates curated and edited by
[fugleramme](https://github.com/arnegiacomo/fugleramme), CC BY-SA 4.0 (sources in each style's
`ATTRIBUTION.md`).*

## Pages

Besides today's birds, a frame can show the week, today's rarest birds, the day's story and the
year so far (`view=`), each frame its own layout and page (`FRAMES`). A bird new for the year is
drawn first and biggest, and frames wake every 5 minutes while it is about. The bird pages end
with a fun fact about their main bird (from a table built in, no cloud), and every page wears an
engraved seasonal border: frost, blossom, a summer meadow, autumn leaves, and holly, Easter
feathers or a Midsummer garland on those holidays. On every ESP32 frame the previous and next
buttons step through all five pages. Frames report battery and last wake to Home Assistant. See [Pages and views](docs/explanation/pages-and-views.md).

| The day so far | The year so far |
| --- | --- |
| ![The story page: first and last singers, the weather, and when each bird sang](docs/images/view-story.png) | ![The season page: species this year, arrivals, this week last year, the year list](docs/images/view-season.png) |

## Quick start (no BirdNET-Go needed)

```bash
BIRDNET_URL=mock cargo run --release -p server
# open http://localhost:8080/
# other pages: http://localhost:8080/preview.png?view=story (or week, rarest, season)
# spring in the simulated garden: MOCK_DATE=2026-05-01
```

`BIRDNET_URL=mock` uses the built-in simulated station: a garden of 35 European species with a
dawn chorus, deterministic for a given time. The collage needs fugleramme's plates: fetch them
once (about 250 MB, never committed here) and point `ART_DIR` at them. Until then the birds are
listed by name.

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
3. **The firmware** (`apps/ee02-thin`) runs on a Seeed XIAO EE02 (ESP32-S3) board. It wakes on
   a timer or a button press, rejoins the Wi-Fi it remembers, and sends a conditional
   `GET /panel` with its battery voltage. It refreshes the panel only if the image changed, then
   deep-sleeps for the server's `max-age`. It is set up from a phone over its own hotspot,
   installs new firmware the server offers, and its buttons redraw the page or step between
   today, this week, the rarest bird, the day's story and the season
   ([how](docs/how-to/set-up-ee02-over-wifi.md)).

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

The server also serves the 7.3" Spectra 6 panel (the thin firmware runs on Seeed's reTerminal
E1002 with it) and PNGs at any size. Without a server, a Raspberry Pi
([how](docs/how-to/run-on-raspberry-pi.md)) or an ESP32
([how](docs/how-to/flash-ee02-standalone.md)) can draw the page itself. See
[Supported hardware](docs/reference/supported-hardware.md) for which panels, ESP32 models and
combinations work today.

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
| `crates/art-webp` | fugleramme's art folders: a plate per species per day, paper halo toned and feathered | Done |
| `crates/page-text` | `no_std` + `alloc`: the page's words in a dozen languages, from a translations JSON | Done |
| `crates/bird-sizes` | `no_std`: AVONET body masses for the 6,421 species BirdNET detects | Done |
| `crates/bird-facts` | `no_std`: a few facts per species from AVONET and Wikidata, one told a day | Done |
| `crates/frame-layout` | `no_std` + `alloc`: what goes where; the collage's spiral packing by outline, out to the page's corners; the story chart and season page; the seasonal borders | Collage, list, story, season and borders done |
| `crates/bird-almanac` | `no_std` + `alloc`: new birds for the year, arrivals, the rarest, year on year, the day's story | Done |
| `crates/open-meteo` | `no_std` + `alloc`: today's weather from Open-Meteo, keyless | Done |
| `crates/home-assistant` | Frames as Home Assistant devices over MQTT discovery, with a small MQTT publisher | Done |
| `crates/frame-raster` | `no_std` + `alloc`: draws pictures and text, boosts and dithers to six inks | Done |
| `crates/frame-http` | Serves `/panel`, `/frame.png`, `/preview.png` per frame and view, `/api/frames` and firmware; a small zlib compressor | Done |
| `apps/server` | The server for Unraid / Docker | Done |
| `apps/fake-birdnet` | The simulated BirdNET-Go over HTTP | Done |
| `crates/panel-t133a01` | `no_std`: the 13.3" panel's two controllers over `embedded-hal` 1.0, tested against a mock bus | Done; untried on hardware |
| `crates/panel-wake` | `no_std` + `alloc`: each wake's request, unpacking and checking the page, sleep and back-off, the RTC memory and Wi-Fi rejoin record, button presses, battery, and checking firmware updates | Done; tested against the real server |
| `crates/panel-setup` | `no_std` + `alloc`: the setup hotspot's settings record, page and form, and its DHCP and DNS answers | Done; tested on the host |
| `crates/transport-esp` | `no_std` + `alloc`: ESP32 Wi-Fi with fast rejoin and one HTTP GET, and the setup hotspot, on `esp-radio` and `embassy-net` | Builds for the ESP32-S3; untried on hardware |
| `apps/ee02-thin` | ESP32-S3 firmware: `panel-wake`, `panel-setup`, `transport-esp` and `glass-esp` on the EE02 or the reTerminal E1002 (7.3"); deep sleep between wakes, setup over Wi-Fi, over-the-air updates, buttons | Builds for the ESP32-S3; untried on hardware |
| `crates/frame-artist` | `no_std` + `alloc`: draws the page for a list of birds, shared by the server and the frames that draw their own | Done |
| `crates/frame-standalone` | `no_std` + `alloc`: a frame that draws its own page: whether the birds changed, asking BirdNET-Go, its settings, RTC memory and sleep | Done; tested on the host |
| `crates/panel-ed2208` | `no_std`: the 7.3" panel's controller over `embedded-hal` 1.0, tested against a mock bus | Done; untried on hardware |
| `crates/glass-esp` | Each ESP32-S3 board's panel pins, power and deep-sleep pin holds, for the EE02 and the reTerminal E1002 | Builds; untried on hardware |
| `tools/art-packer` | Packs fugleramme's plates into one bundle, about 30 KB a species, for an ESP32's flash | Done |
| `apps/ee02-standalone` | ESP32-S3 firmware with no server: asks BirdNET-Go and draws the page itself, art in flash, buttons stepping through all five pages; EE02 or reTerminal E1002 | Builds for both boards; untried on hardware |
| `apps/linux-panel` | A Raspberry Pi driving the panel over SPI, with no server | Done; tested on a mock bus, untried on a panel |
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
| `ART_DIR` | unset | fugleramme art style folder, e.g. `art/classic`; unset lists the birds by name |
| `LAYOUT` | `collage` | `collage` (pictures packed by outline, sized by body mass) or `list` (names and counts) |
| `MAX_BIRDS` | `0` | Most birds on a page; `0` for every bird heard |
| `NAMES` | `true` | Whether the collage writes each bird's name under it |
| `HEADING` | unset | A heading over the page; the collage has none by default, like fugleramme |
| `FACTS` | `true` | Whether today's, the week's and the rarest page end with a line about their main bird |
| `BORDER` | `season` | The engraved border: `season` (by date and holiday), `off`, or one theme, e.g. `frost` |
| `LANGUAGE` | `en` | Language of the page's own words, e.g. `sv`, `nb`, `de` (bird names come from BirdNET-Go) |
| `TRANSLATIONS` | unset | A translations JSON adding languages or overriding words |
| `VIEW` | `today` | Page for frames that do not ask: `today`, `week`, `rarest`, `story` or `season` |
| `FRAMES` | unset | Frames with their own settings, e.g. `hall:layout=collage;desk:layout=list,panel=7.3,view=story` |
| `LATITUDE`, `LONGITUDE` | unset | Where the garden is, for the weather and sunrise on the story page (Open-Meteo, no key) |
| `ALERT_POLL_MINUTES` | `5` | How long frames sleep while a bird new for the year is about; `0` for no change |
| `MQTT_URL` | unset | `mqtt://user:password@host:port`: announce frames' battery and last wake to Home Assistant |
| `FIRMWARE_DIR` | unset | Folder with `ee02-thin.bin` and `ee02-thin.version`, offered to EE02 frames over Wi-Fi |
| `MOCK_DATE` | unset | With `BIRDNET_URL=mock`, pretend it is another day, e.g. `2026-05-01` |

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
(without the two art lines the birds are listed by name).

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
version, a bare-metal build of the `no_std` crates, the ESP32-S3 firmware, docs and `cargo deny`. Pushing a `v*` tag
builds the release binaries and the container image.

The firmware uses Espressif's Rust toolchain and builds from its own folder, which has its own
workspace and lock file so the commands above never need that toolchain:

```bash
espup install --targets esp32s3 --toolchain-version 1.98.1.0 && . ~/export-esp.sh
cd apps/ee02-thin
cargo run --release    # then set it up from a phone: join BirdFrame-xxxx
```

See [Build and flash the EE02 firmware](docs/how-to/flash-ee02-thin.md) and
[the firmware reference](docs/reference/firmware.md). CI builds it and runs clippy on it on every
push; the crates it builds on (`panel-t133a01`, `panel-wake`, `panel-setup`) also build for bare
metal and test on any machine.

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
  AVONET's ranges, migration, diet and lifestyle, with wingspans, incubation and lifespans from
  **[Wikidata](https://www.wikidata.org/)** (CC0), make the fun facts (`crates/bird-facts/data/`).
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
