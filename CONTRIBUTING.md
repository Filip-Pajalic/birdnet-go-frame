# Contributing

Thanks for helping. Issues and pull requests are welcome, including from forks: CI runs on
every pull request without needing any secrets.

## Getting started

```sh
git clone https://github.com/Filip-Pajalic/birdnet-go-frame
cd birdnet-go-frame
BIRDNET_URL=mock cargo run -p server     # then open http://localhost:8080
cargo test --workspace
```

No microphone, BirdNET-Go or panel is needed: `BIRDNET_URL=mock` simulates a garden, and
`cargo run -p fake-birdnet` serves the same simulation over HTTP. See
[docs/how-to/develop-and-test.md](docs/how-to/develop-and-test.md).

## Before you open a pull request

```sh
cargo +nightly fmt --all                               # Google style options need nightly rustfmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- Follow [STYLE.md](STYLE.md). It is Google's Rust style guide, applied strictly.
- Libraries stay `no_std` with `alloc` unless their job needs an operating system, so the
  ESP32 can use them. CI builds them for a bare-metal target to check.
- Keep dependencies minimal. A new third-party crate needs a reason good enough to write in
  [docs/explanation/dependencies.md](docs/explanation/dependencies.md); `cargo deny` checks
  licences, advisories and duplicates.
- Add or update tests with the change. The E6 golden tests must keep passing byte for byte.
- Update the docs in `docs/` when settings or behaviour change.

## Licensing

Contributions are accepted under the MIT licence of this repository. Do not copy code from
BirdNET-Go (CC BY-NC-SA) and do not commit fugleramme's artwork (CC BY-SA); see
[NOTICE.md](NOTICE.md).

## Releases (maintainers)

Bump `version` in `Cargo.toml`, commit, then push a tag:

```sh
git tag v0.2.0 && git push origin v0.2.0
```

The release workflow builds Linux (x86_64, aarch64, static), macOS and Windows binaries,
attaches them with checksums to a GitHub release, and the container workflow publishes
`ghcr.io/filip-pajalic/birdnet-go-frame:0.2.0` and `:latest`-style tags. Tags with a hyphen
(`v0.2.0-rc.1`) become pre-releases.
