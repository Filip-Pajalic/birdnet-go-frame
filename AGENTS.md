# AGENTS.md

Guidance for AI coding agents (and humans) working in this repository.

## What this is

birdnet-go-frame renders the birds heard by a [BirdNET-Go](https://github.com/tphakala/birdnet-go)
station onto a 13.3" Spectra 6 e-ink panel. A Rust server reads BirdNET-Go's HTTP API and serves
a finished page; an ESP32 fetches it, redraws only on change, and deep-sleeps. It is a Rust
re-implementation of the server/panel idea from
[fugleramme](https://github.com/arnegiacomo/fugleramme) (MIT), and its `/frame.e6` output is
byte-compatible with fugleramme's.

Read [`docs/README.md`](docs/README.md) first; [`docs/explanation/architecture.md`](docs/explanation/architecture.md)
and [`docs/explanation/image-pipeline.md`](docs/explanation/image-pipeline.md) explain the moving parts.

## Layout

- `crates/` holds libraries only. `apps/` holds thin binaries that wire libraries together and
  read configuration; no logic belongs there that a library could own.
- Libraries are `no_std` + `alloc` unless their job needs an OS (`transport-ureq`, `frame-http`).
  Keep it that way: the same code must compile for the ESP32. Do not add `std` to `frame-core`,
  `birdnet-model`, `birdnet-client`, `birdnet-mock`, `frame-layout` or `frame-raster`.
- Platform differences go behind small generic traits (`HttpTransport`, later `embedded-hal`),
  not `cfg` branches inside libraries. Prefer generics over `dyn`.
- See [`docs/reference/crates.md`](docs/reference/crates.md) for what each crate owns.

## Commands

```bash
cargo +nightly fmt --all                                   # formatting (nightly-only rustfmt options)
cargo clippy --workspace --all-targets -- -D warnings      # lints, must be clean
cargo test --workspace                                     # all tests
BIRDNET_URL=mock cargo run -p server                       # run against the simulated station
cargo run -p fake-birdnet -- 127.0.0.1:8090                # simulated BirdNET-Go over HTTP
```

Run fmt, clippy and tests before every commit; CI runs the same with `--locked`.

## Code style

Follow [Google's Rust Style Guide](https://google.github.io/styleguide/rustguide.html) strictly.
[`STYLE.md`](STYLE.md) lists what tooling enforces and the review checklist. The points agents most
often get wrong:

- One `use` per module path, no nested `::` in braces, no glob imports (except `use super::*` in
  test modules), no preludes. Types and traits used unqualified; free functions called through
  their module (`e6::encode`, `frame_raster::dither`).
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

- Use `birdnet-mock` (`MockTransport` with a fixed clock) instead of network calls in tests.
- Add a JSON fixture under `crates/birdnet-model/tests/fixtures/` when BirdNET-Go sends a shape
  the parser mishandles.
- `crates/frame-core/tests/e6_golden.rs` guards byte compatibility with fugleramme. Never update
  its expected bytes to make a change pass; a failure there means the panel format broke.
- Test files may panic on failed assumptions and start with `#![allow(clippy::expect_used)]`.

## Licensing rules

- Code is MIT. Code ported from fugleramme keeps its notice: say so in the file's doc comment and
  keep [`NOTICE.md`](NOTICE.md) accurate.
- Never commit fugleramme's bird artwork (CC BY-SA 4.0, about 245 MB). It is downloaded at install
  time with its `ATTRIBUTION.md` and `manifest.json`.
- Fonts go under `assets/fonts/<family>/` with their `OFL.txt`, listed in
  `assets/fonts/ATTRIBUTION.md`.
- Do not copy code from BirdNET-Go (CC BY-NC-SA 4.0); only call its HTTP API.

## Docs

Docs live in `docs/` and follow [Diátaxis](https://diataxis.fr/): tutorials, how-to guides,
reference, explanation. Put a new page in the one section that matches its purpose, link it from
`docs/README.md`, and use Mermaid for diagrams. When behaviour, configuration or an endpoint
changes, update the matching reference page in the same commit.

## Commits

Small, focused commits with an imperative subject line. Do not commit `target/` or generated
art. Keep `Cargo.lock` committed.
