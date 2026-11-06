# Run an ESP32 frame with no server

`apps/ee02-standalone` is firmware for an ESP32-S3 frame that asks BirdNET-Go itself and draws
its own page: the same collage or list the server draws, with the art built into the firmware.
There is no frame server to run. It suits one frame on a BirdNET-Go you cannot or would rather
not run a container beside. It runs on two boards:

| Board | Panel | Build |
| --- | --- | --- |
| Seeed XIAO ePaper Display Board EE02 | 13.3" Spectra 6, 1600x1200 | the default |
| Seeed reTerminal E1002 | 7.3" Spectra 6, 800x480 | `--no-default-features --features board-e1002` |

What it gives up against [the thin firmware](flash-ee02-thin.md) and the server: no setup
hotspot (the settings are built in), no over-the-air updates, no buttons or other pages (the
story and season pages need the server's longer history), no weather, no Home Assistant, and no
login to a private BirdNET-Go. The 13.3" page is drawn at half size and enlarged when it is
dithered, so it is a little softer than the server's.

> The firmware compiles in CI and its logic and drawing are tested on a laptop, but it has not
> yet run on either board. Please open an issue with the serial log from your first wakes.

## You need

- The board with its panel, and a USB-C cable
- BirdNET-Go reachable from your Wi-Fi at a fixed IPv4 address, with its API open (no
  private mode)
- Espressif's Rust toolchain and `espflash`, installed as in step 1 of
  [Build and flash the EE02 firmware](flash-ee02-thin.md#1-install-espressifs-toolchain)

## 1. Pack the art

Without art the frame has no pictures and lists the birds by name. For fugleramme's plates, fetch a style and pack it,
from the repository's root:

```bash
scripts/fetch-art.sh classic art               # into art/classic, never committed
cargo run --release -p art-packer -- art/classic frame-art.artb --species my-birds.txt
```

`my-birds.txt` lists one scientific name a line, for example the species in your BirdNET-Go's
last year. Each species takes about 30 KB, and the firmware has room for about 5 MB of art, so
about 150 species; without `--species` every species in the style is packed, which may not fit.
`--day N` picks which plate of a species with several (the frame shows the same plate every
day, unlike the server).

## 2. Set the settings and build

```bash
export FRAME_WIFI_SSID="your-network"
export FRAME_WIFI_PASSWORD="your-password"     # empty for an open network
export FRAME_BIRDNET="192.168.1.10:8080"       # BirdNET-Go's address and port
export FRAME_ART="$PWD/frame-art.artb"         # optional: the bundle from step 1
cd apps/ee02-standalone
cargo run --release                            # EE02: builds, flashes, shows the serial log
# or, for the reTerminal E1002:
cargo run --release --no-default-features --features board-e1002
```

The rest are optional:

| Variable | Default | What it does |
| --- | --- | --- |
| `FRAME_ROTATION` | `0` | `0`, `90`, `180` or `270`: how the panel hangs |
| `FRAME_LAYOUT` | `collage` | `collage` or `list` |
| `FRAME_LANGUAGE` | `en` | The page's own words, as `LANGUAGE` on the server |
| `FRAME_UTC_OFFSET_MINUTES` | `0` | Your time zone, minutes east of UTC (`60` for Oslo in winter); the frame has no time zone table, so change it with daylight saving or live with an hour |
| `FRAME_POLL_MINUTES` | `15` | Minutes between wakes |
| `FRAME_WINDOW_HOURS` | `24` | How far back detections count |
| `FRAME_MAX_BIRDS` | `12` | Most birds on the page; each one's picture is held while drawing |
| `FRAME_NAMES` | `true` | Whether the collage names its birds |

A password built in ends up in the firmware image, so treat the built file like the password.

## 3. Check the first wakes

```text
standalone: firmware 0.1.0 for the XIAO EE02 and 13.3" panel
standalone: 3 species heard; drawing
standalone: 148 species in the art bundle
standalone: drew the page in 24310 ms
standalone: refreshed in 28431 ms
standalone: sleeping 900 s
```

Each wake asks BirdNET-Go for the time (from its `Date` header; the frame has no clock that
survives deep sleep) and for the birds heard in the window. When the page would look the same
as the one on the glass, it goes straight back to sleep: `the page has not changed`.

## If something goes wrong

| Log line | Fix |
| --- | --- |
| `FRAME_WIFI_SSID is missing or cannot be read` | Set it and build again; the frame sleeps an hour and tries again |
| `FRAME_BIRDNET is missing or cannot be read` | It must be an IPv4 address with an optional port, not a host name |
| `could not join the Wi-Fi` | Check the name and password, and that the network has 2.4 GHz |
| `BirdNET-Go: BirdNET-Go wants a login; ...` | BirdNET-Go's private mode is on; this firmware cannot log in, so turn it off or use the server |
| `BirdNET-Go's answer has no Date header` | Something between the frame and BirdNET-Go drops it; reach BirdNET-Go directly |
| `the art bundle: ...` | `FRAME_ART` is not a bundle from `art-packer`; the frame lists the birds by name |

After a failure the picture stays and the frame tries again in a minute, doubling after each
failure in a row up to the poll interval.
