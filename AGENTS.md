# AGENTS.md

Guidance for AI coding agents (and humans) working in this repository.

## What this is

birdnet-go-frame renders the birds heard by a [BirdNET-Go](https://github.com/tphakala/birdnet-go)
station onto a 13.3" Spectra 6 e-ink panel. A Rust server reads BirdNET-Go's HTTP API and serves
a finished page; an ESP32 fetches it, redraws only on change, and deep-sleeps. It is a Rust
re-implementation of the server/panel idea from
[fugleramme](https://github.com/arnegiacomo/fugleramme) (MIT); the pixel bytes `/panel` serves
are those of fugleramme's `/frame.e6`, without its header and zlib-compressed.

Read [`docs/README.md`](docs/README.md) first; [`docs/explanation/architecture.md`](docs/explanation/architecture.md)
and [`docs/explanation/image-pipeline.md`](docs/explanation/image-pipeline.md) explain the moving parts.

## Layout

- `crates/` holds libraries only. `apps/` holds thin binaries that wire libraries together and
  read configuration; no logic belongs there that a library could own.
- Libraries are `no_std` + `alloc` unless their job needs an OS (`lan-http`, `transport-lan`,
  `frame-http`, `art-webp`). Keep it that way: the same code must compile for the ESP32, and CI builds those
  crates for a bare-metal target. Do not add `std` to `frame-core`, `birdnet-model`,
  `birdnet-client`, `birdnet-mock`, `lite-json`, `art-pack`, `art-mock`, `bird-sizes`,
  `page-text`, `frame-layout`, `frame-raster`, `panel-t133a01` or `panel-wake`. `art-webp` is `std` (it reads folders) and is the
  only crate that decodes images.
- The ESP32 firmware (`apps/ee02-thin`) and its radio crate (`crates/transport-esp`, `no_std`)
  build only with Espressif's Xtensa toolchain, so they sit in their own workspace in
  `apps/ee02-thin` with their own `Cargo.lock`, excluded from the root one. Keep everything that
  can be tested on a laptop out of them: decisions, parsing and byte layouts belong in
  `panel-wake`, and `transport-esp` and the app only move bytes and drive the chip. They repeat
  the workspace's lints in their own `Cargo.toml`; keep those tables in step.
- Platform differences go behind small generic traits (`HttpTransport`, `ArtSource`,
  `embedded-hal`'s SPI, pin and delay traits),
  not `cfg` branches inside libraries. Prefer generics over `dyn`.
- See [`docs/reference/crates.md`](docs/reference/crates.md) for what each crate owns.

## Dependencies

The server has eleven third-party crates (`fontdue` and `image-webp` with their own); the panel
side adds `embedded-hal`. Do not add one without a strong reason: read
[`docs/explanation/dependencies.md`](docs/explanation/dependencies.md), turn off default
features, add a row there, and make sure `cargo deny check` passes. The firmware's crates have
their own table there; prefer what esp-hal and embassy already bring. Small jobs (JSON, HTTP, PNG,
zlib both ways) are done in-repo on purpose.

## Commands

```bash
cargo +nightly fmt --all                                   # formatting (nightly-only rustfmt options)
cargo clippy --workspace --all-targets -- -D warnings      # lints, must be clean
cargo test --workspace                                     # all tests
BIRDNET_URL=mock cargo run -p server                       # run against the simulated station
cargo run -p fake-birdnet -- 127.0.0.1:8090                # simulated BirdNET-Go over HTTP

# Firmware, after `espup install --targets esp32s3 --toolchain-version 1.98.1.0`:
. ~/export-esp.sh
cd apps/ee02-thin
export FRAME_WIFI_SSID=ci FRAME_WIFI_PASSWORD=ci-password FRAME_SERVER=192.0.2.1:8080
cargo +nightly fmt --check && (cd ../../crates/transport-esp && cargo +nightly fmt --check)
cargo clippy --release --locked -- -D warnings
cargo build --release --locked
```

Run fmt, clippy and tests before every commit; CI runs the same with `--locked`, plus tests on
Linux, macOS and Windows, the minimum Rust version (1.87, set by fontdue), a bare-metal build of
the `no_std` crates, `cargo doc` and `cargo deny`, and builds the firmware with the Xtensa
toolchain. The firmware cannot be tested on the host; prove its logic with tests in `panel-wake`.

## Code style

Follow [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html) strictly.
[`STYLE.md`](STYLE.md) lists what tooling enforces and the review checklist. The points agents most
often get wrong:

- One `use` per module path, no nested `::` in braces, no glob imports (except `use super::*` in
  test modules), no preludes. Types and traits used unqualified; free functions called through
  their module (`panel_bytes::pack`, `frame_raster::dither`).
- `lib.rs` contains only `mod` declarations and re-exports. One file per abstraction, named after
  its main item.
- No `unwrap()`. `expect("why")` only for true invariants, marked with
  `#[expect(clippy::expect_used, reason = "...")]`. Errors are enums or structs implementing
  `Display` and `Error`, marked `#[must_use]`.
- Explicit types on `let` for `.parse()`, `.into()`, `collect()` and anything non-obvious.
- Every public item has a `///` doc comment in full sentences, with `# Errors` / `# Panics`
  sections where they apply. No comments that restate the code.
- No `unsafe` (it is forbidden workspace-wide). No new proc macros; no code before `main()`.

## Testing

- Use `birdnet-mock` (`MockTransport` with a fixed clock) instead of network calls in tests, and
  `art-mock` instead of image files. Never commit fugleramme's artwork (CC BY-SA, ~250 MB):
  `scripts/fetch-art.sh` downloads it into the git-ignored `art/` folder, and credit for it lives
  in README and NOTICE only, never on the rendered page.
- Add a JSON fixture under `crates/birdnet-model/tests/fixtures/` when BirdNET-Go sends a shape
  the parser mishandles.
- `crates/frame-core/tests/panel_bytes_golden.rs` guards byte compatibility with fugleramme. Never update
  its expected bytes to make a change pass; a failure there means the panel format broke.
- Test files may panic on failed assumptions and start with `#![allow(clippy::expect_used)]`.

## Licensing rules

- Code is MIT. Code ported from fugleramme keeps its notice: say so in the file's doc comment and
  keep [`NOTICE.md`](NOTICE.md) accurate.
- Never commit fugleramme's bird artwork (CC BY-SA 4.0, about 245 MB). It is downloaded at install
  time with its `ATTRIBUTION.md` and `manifest.json`.
- Fonts go under `assets/fonts/<family>/` with their `OFL.txt`, listed in
  `assets/fonts/ATTRIBUTION.md`. Data files sit with the crate that embeds them, with an
  `ATTRIBUTION.md` (e.g. `crates/bird-sizes/data/`).
- Do not copy code from BirdNET-Go (CC BY-NC-SA 4.0); only call its HTTP API.

## Docs

Docs live in `docs/` and follow [Diátaxis](https://diataxis.fr/): tutorials, how-to guides,
reference, explanation. Put a new page in the one section that matches its purpose, link it from
`docs/README.md`, and use Mermaid for diagrams. When behaviour, configuration or an endpoint
changes, update the matching reference page in the same commit.

## Releases and CI

Workflows are in `.github/workflows/`: `ci.yml` (every push to main and every pull request,
fork-safe, read-only), `release.yml` (on `v*` tags: binaries for Linux, macOS and Windows with
checksums on a GitHub release), `docker.yml` (image to `ghcr.io` from main and tags). Never
switch a workflow to `pull_request_target` or give PR jobs write permissions or secrets.

## Commits

Small, focused commits with an imperative subject line. Do not commit `target/` or generated
art. Keep `Cargo.lock` committed.
