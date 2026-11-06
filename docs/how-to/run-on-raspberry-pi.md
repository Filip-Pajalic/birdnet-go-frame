# Run the frame on a Raspberry Pi, with no server

`apps/linux-panel` (binary `birdnet-go-frame-panel`) is the whole frame in one program on a
Raspberry Pi: it asks BirdNET-Go for the birds, draws the page exactly as the server would, and
shows it on a Spectra 6 panel wired to the Pi's SPI bus. There is no server and no ESP32. It is
the right choice when the frame hangs near a power socket; for a battery frame, use an ESP32
(see [Supported hardware](../reference/supported-hardware.md)).

> The app is tested against a mock SPI bus and the simulated BirdNET-Go, and draws pages into a
> file on any computer, but it has not yet driven a real panel. Please open an issue with what
> you see.

## You need

- A Raspberry Pi with a 40-pin header (any model with Wi-Fi or Ethernet; a Zero 2 W is plenty)
  running Raspberry Pi OS, 64-bit for the release binary
- A 13.3" Spectra 6 panel (two controllers, 1600x1200) or a 7.3" one (800x480, such as
  Waveshare's 7.3" e-Paper (E)) with an SPI driver board or HAT
- BirdNET-Go reachable from the Pi

## 1. Wire the panel

The defaults are the pins of Waveshare's e-Paper HAT+ boards, by BCM number. Other boards work
with the `PIN_*` settings below.

| Signal | BCM pin | Header pin | Setting |
| --- | --- | --- | --- |
| SPI clock | 11 (SCLK) | 23 | |
| SPI data | 10 (MOSI) | 19 | |
| Chip select (first controller) | 8 | 24 | `PIN_CS0` |
| Chip select (13.3" second controller) | 7 | 26 | `PIN_CS1` |
| Data/command | 25 | 22 | `PIN_DC` |
| Reset | 17 | 11 | `PIN_RST` |
| BUSY | 24 | 18 | `PIN_BUSY` |
| Panel power switch, if the board has one | 18 | 12 | `PIN_POWER` (`none` for no switch) |

The app drives the chip selects itself, because the 13.3" panel needs both controllers
selected at once for some commands. So the kernel must not claim pins 8 and 7 as SPI chip
selects: turn SPI on without them in `/boot/firmware/config.txt` and reboot:

```ini
dtparam=spi=on
dtoverlay=spi0-0cs
```

`ls /dev/spidev0.0 /dev/gpiochip0` should then show both.

## 2. Install the program

Download `birdnet-go-frame-<version>-aarch64-unknown-linux-musl.tar.gz` from the
[releases](https://github.com/Filip-Pajalic/birdnet-go-frame/releases) and copy
`birdnet-go-frame-panel` to `/usr/local/bin/`. Or build it on the Pi:

```bash
cargo build --release -p linux-panel
sudo cp target/release/birdnet-go-frame-panel /usr/local/bin/
```

## 3. Try it without the panel

`PANEL_OUT` writes the panel's bytes to a file instead of the glass, and `--once` draws one page
and exits, so you can check your settings before any wiring:

```bash
BIRDNET_URL=http://192.168.1.10:8080 PANEL_OUT=/tmp/page.bin birdnet-go-frame-panel --once
ls -l /tmp/page.bin    # 960000 bytes for the 13.3" panel, 192000 for the 7.3"
```

`BIRDNET_URL=mock` draws birds from the simulated station.

## 4. Run it for good

Put the settings in `/etc/birdnet-go-frame-panel.env`:

```ini
BIRDNET_URL=http://192.168.1.10:8080
PANEL=13.3
ROTATION=0
UTC_OFFSET_MINUTES=60
LANGUAGE=sv
# ART_DIR=/opt/birdnet-go-frame/art/classic   # from scripts/fetch-art.sh; a list without
```

and install the service from `deploy/raspberry-pi/`:

```bash
sudo useradd --system --groups spi,gpio birdnet-frame
sudo cp deploy/raspberry-pi/birdnet-go-frame-panel.service /etc/systemd/system/
sudo systemctl enable --now birdnet-go-frame-panel
journalctl -u birdnet-go-frame-panel -f    # "drew 12 species", "refreshed in 28000 ms"
```

## Settings

Every page setting has the server's name and meaning (see
[Configuration](../reference/configuration.md)): `BIRDNET_URL`, `BIRDNET_PASSWORD`,
`BIRDNET_USERNAME`, `PANEL`, `ROTATION`, `WINDOW_HOURS`, `UTC_OFFSET_MINUTES`, `LAYOUT`,
`MAX_BIRDS`, `NAMES`, `HEADING`, `LANGUAGE`, `TRANSLATIONS` and `ART_DIR`. The rest are the
Pi's own:

| Variable | Default | Meaning |
| --- | --- | --- |
| `POLL_MINUTES` | `15` | Minutes between asking BirdNET-Go. The panel refreshes only when the page changes, and at most this often |
| `PANEL_OUT` | unset | Write each new page's panel bytes to this file instead of driving the panel |
| `SPI_DEVICE` | `/dev/spidev0.0` | The SPI device |
| `SPI_HZ` | `10000000` for 13.3", `4000000` for 7.3" | The SPI clock |
| `GPIO_CHIP` | `/dev/gpiochip0` | The GPIO chip the pins are on. On a Pi 5 with a kernel older than 6.6.45, use `/dev/gpiochip4` |
| `PIN_CS0`, `PIN_CS1`, `PIN_DC`, `PIN_RST`, `PIN_BUSY` | `8`, `7`, `25`, `17`, `24` | Line numbers on that chip |
| `PIN_POWER` | `18` | The panel's power switch, driven high while the panel works; `none` when the board has none |

## When something is wrong

- **`/dev/spidev0.0: No such file or directory`**: SPI is off; see step 1.
- **`GPIO line 8: ... busy`**: the kernel holds the pin as a chip select; add
  `dtoverlay=spi0-0cs` and reboot.
- **`panel stayed busy`** or **`panel did not start refreshing`**: check BUSY's and reset's
  wiring, and the power pin; a panel that never gets power stays busy.
- **`panel released BUSY after only ... ms`**: the refresh started but stopped early, which
  usually means the panel's supply is too weak.
