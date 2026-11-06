# Develop and test without a microphone

## Run everything

```bash
cargo +nightly fmt --all                     # Google style options need nightly rustfmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Use the simulated BirdNET-Go

In-process, no network:

```bash
BIRDNET_URL=mock cargo run -p server
```

Over HTTP, for testing the real client and transport, or another frame such as fugleramme:

```bash
cargo run -p fake-birdnet -- 127.0.0.1:8090
BIRDNET_URL=http://127.0.0.1:8090 LISTEN=127.0.0.1:8081 cargo run -p server
```

Set `UTC_OFFSET_MINUTES` on `fake-birdnet` to move its dawn chorus to your time zone.

## In tests

`MockTransport` plugs the simulated station into the client with a fixed clock, so results never
change between runs:

```rust
use birdnet_client::{BirdnetClient, DateWindow};
use birdnet_mock::MockTransport;

let now: i64 = 1_793_955_600; // 2026-11-06T09:00:00Z
let mut client = BirdnetClient::new("mock", MockTransport::new(move || now));
let species = client.species_summary(DateWindow::hours_back(now, 0, 24))?;
```

For a response the simulator does not produce, implement `HttpTransport` on a small struct that
returns canned bytes; the example in `crates/birdnet-client/src/lib.rs` shows one.

## Recorded responses

`crates/birdnet-model/tests/fixtures/` holds JSON in BirdNET-Go's shapes. Add a fixture there
when a real station sends something the parser gets wrong.

## Checking the panel format

`crates/frame-core/tests/e6_golden.rs` compares our `/frame.e6` with bytes from a reference
implementation of fugleramme's packing. Any change that breaks byte compatibility fails it.
