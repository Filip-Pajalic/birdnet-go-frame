## What and why

<!-- What does this change, and why? Link the issue if there is one. -->

## How it was checked

<!-- Tests added or run, and anything checked by hand (e.g. the page rendered with BIRDNET_URL=mock). -->

## Checklist

- [ ] `cargo +nightly fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass
- [ ] Follows [STYLE.md](../STYLE.md) (Google's Rust style)
- [ ] No new third-party crate, or the reason is given above and in `docs/explanation/dependencies.md`
- [ ] Docs updated if behaviour or settings changed
- [ ] No BirdNET-Go code or fugleramme artwork copied (see [NOTICE.md](../NOTICE.md))
