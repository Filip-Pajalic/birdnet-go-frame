# Build and flash the EE02 firmware

`apps/ee02-thin` is birdnet-go-frame's own firmware for the Seeed XIAO EE02 with the 13.3"
Spectra 6 panel. It reads `/panel` directly, redraws only on a change, and sleeps for as long as
the server's `Cache-Control: max-age` says. This guide builds it and puts it on the board.

> The firmware compiles and its logic is tested on a laptop, but it has not yet run on an EE02.
> Please open an issue with the serial log from your first wakes, good or bad.

## You need

- The EE02 with the panel connected, and a USB-C cable
- A server running and reachable from your Wi-Fi (see
  [Connect to your BirdNET-Go](connect-birdnet-go.md)), at a fixed IPv4 address
- Linux or macOS with Rust installed through `rustup` (Windows works with the same commands in
  PowerShell, setting the variables with `$env:NAME = "value"`)

## 1. Install Espressif's toolchain

The ESP32-S3 is an Xtensa chip, which needs Espressif's fork of the Rust compiler. `espup`
installs it next to your normal toolchain as one named `esp`:

```bash
cargo install espup --locked     # or download a release binary from github.com/esp-rs/espup
espup install --targets esp32s3 --toolchain-version 1.98.1.0
. ~/export-esp.sh                # puts the Xtensa linker on PATH; do this in every new shell
```

Use 1.98.1.0: version 1.99.0.0 links this firmware wrongly ("dangerous relocation: l32r: literal
placed after use"). `apps/ee02-thin/rust-toolchain.toml` picks the `esp` toolchain for you.

Install the flasher too:

```bash
cargo install espflash --locked  # needs libudev-dev on Debian/Ubuntu; or use a release binary
```

## 2. Set the build-time settings

The settings are compiled in, so nothing on the board needs configuring:

```bash
export FRAME_WIFI_SSID="your-network"
export FRAME_WIFI_PASSWORD="your-password"     # empty for an open network
export FRAME_SERVER="192.168.1.20:8081"        # the address and port the server is published on
export FRAME_ROTATION=90                       # optional: 0, 90, 180 or 270; else the server decides
```

See [the firmware reference](../reference/firmware.md) for what each one does. The password ends up
in the firmware image, so treat the built file like the password itself.

## 3. Build and flash

With the board plugged in:

```bash
cd apps/ee02-thin
cargo run --release              # builds, flashes with espflash, then shows the serial log
```

If the board does not show up as a serial port, hold **BOOT**, press **RESET**, release
**BOOT**, and run the command again.

To build without flashing, use `cargo build --release`; the image is
`target/xtensa-esp32s3-none-elf/release/ee02-thin`.

## 4. Check the first wakes

The log shows each wake in a few lines:

```text
ee02: 200 with 214873 bytes, max-age Some(900)
ee02: refreshed in 28431 ms
ee02: sleeping 900 s
```

The first wake scans for the network and asks DHCP for an address, then draws the page (about 30
seconds of refresh). Later wakes go straight to the same access point with the same address and
usually end at `ee02: 304 with 0 bytes`, without touching the panel.

The board sleeps between wakes, so its serial port disappears. To see a wake again, press
**RESET**. The page's ETag and the Wi-Fi details live in RTC memory, which survives deep sleep
but may not survive the reset button or a power cut; losing them costs one slow join and one
redraw, nothing more.

## If something goes wrong

| Log line | Fix |
| --- | --- |
| `ee02: could not join the Wi-Fi` | Check `FRAME_WIFI_SSID` and `FRAME_WIFI_PASSWORD`, and that the network has 2.4 GHz |
| `ee02: fast rejoin failed (...), scanning` | Nothing: the access point or channel changed, and the next wake is fast again |
| `ee02: no address from DHCP` | The router did not answer in 10 s; it retries with a growing pause |
| `ee02: the frame refused the connection` | Check `FRAME_SERVER` and that the server's port is published |
| `ee02: FRAME_SERVER: ...` | `FRAME_SERVER` must be an IPv4 address with an optional port, not a host name |
| `ee02: the page is ... bytes, this panel takes 960000` | The server is set up for another panel size |
| `ee02: panel did not start refreshing` or similar | Check the panel's ribbon cable; the page is fetched again next wake |

After a failure the board keeps the old picture and sleeps 5 minutes, doubling after each further
failure up to 12 hours. One good wake resets that.
