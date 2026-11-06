# The EE02 firmware

`apps/ee02-thin`: firmware for the Seeed XIAO EE02 (ESP32-S3, 8 MB octal PSRAM) and the 13.3"
Spectra 6 panel. [Build and flash the EE02 firmware](../how-to/flash-ee02-thin.md) shows how to
install it, [Set up and update the EE02 over Wi-Fi](../how-to/set-up-ee02-over-wifi.md) how to
configure and update it afterwards, and [Supported hardware](supported-hardware.md) which other
boards and panels work.

> Everything here builds, and the decisions are tested on a laptop, but none of it has run on an
> EE02 yet. The button and battery pins come from Seeed's documentation for the EE04, which says
> the EE0x boards share them; check them against the EE02's schematic before relying on them.

## Settings

The frame keeps its settings in its own flash partition, `frame` (see [Flash layout](#flash-layout)),
as a checksummed record (`panel_setup::FrameSettings`). They are entered on the frame's
[setup hotspot](../how-to/set-up-ee02-over-wifi.md) and survive reflashing, updates and power cuts.

| Setting | Form field | Meaning |
| --- | --- | --- |
| Wi-Fi name | `ssid` | 1 to 32 bytes. 2.4 GHz only. |
| Wi-Fi password | `password`, `open` | WPA2 (or WPA2/WPA3 mixed), 8 to 63 bytes; tick "open network" for none. Left empty, the saved password is kept. The page never shows it. |
| Frame server | `server` | `a.b.c.d:port`, or `a.b.c.d` for port 8080. An `http://` prefix and trailing `/` are accepted. Host names are not: the panel skips DNS on every wake. |
| Frame name | `frame` | Optional; `a-z`, `0-9` and `-`, at most 32. Sent as `frame=`, so the server can give this frame its own layout. |
| Rotation | `rotation` | `0`, `90`, `180` or `270`, sent as `rotation=`; "as the server says" leaves it to the server's `ROTATION`. |

### Build-time fallback

When the partition holds no valid record, the firmware uses settings compiled in from these
variables, if `FRAME_WIFI_SSID` is set; otherwise it opens the setup hotspot. They are read with
`option_env!`, so a build without them is fine.

| Variable | Meaning |
| --- | --- |
| `FRAME_WIFI_SSID` | The Wi-Fi network's name. |
| `FRAME_WIFI_PASSWORD` | The password; unset or empty for an open network. |
| `FRAME_SERVER` | The server, as above. |
| `FRAME_ROTATION` | `0`, `90`, `180` or `270`; unset leaves it to the server. |
| `FRAME_NAME` | The frame name, as above. |

## The setup hotspot

Opened when there are no settings, or when the refresh button is held for 3 seconds at power-up
or reset.

| | |
| --- | --- |
| Network | `BirdFrame-xxxx` (the last two bytes of the access point's MAC), open, at most 4 devices |
| Address | The frame is `192.168.4.1/24`; phones get `.2` to `.9` from its DHCP server, with the frame as gateway and DNS |
| DNS | Every name answers `192.168.4.1` (type A; other types answer empty), so phones show their "sign in to network" prompt |
| HTTP | `GET /` shows the form; `POST /save` checks and saves it; any other path redirects to `http://192.168.4.1/` |
| Networks | Scanned once before the hotspot opens and offered as suggestions for the Wi-Fi name |
| Open for | 10 minutes. A save restarts the frame with the new settings. Without a save, a frame that has settings restarts and uses them; one that has none sleeps until a button is pressed. |

## Buttons

From Seeed's PlatformIO cookbook for the EE0x boards, which share the EE04's button wiring. All
three are pulled up, read low when pressed, and wake the frame from deep sleep (`ext1`).

| Button | GPIO | Press | Hold 3 s at power-up or reset |
| --- | --- | --- | --- |
| Refresh (KEY1) | 2 | Wakes now and draws the page even if it has not changed (forgets the ETag) | Opens the setup hotspot |
| Previous (KEY2) | 3 | Steps to the previous view and fetches it | |
| Next (KEY3) | 5 | Steps to the next view and fetches it | |

The views go round in this order and are sent as `view=`; `today` is the server's default and is
not sent. The chosen view is kept in RTC memory until a button changes it. The list is every
page the server draws: a test in `frame-standalone` fails if the two ever differ. The
reTerminal E1002 build has the same three buttons and cycle, and so has the
[standalone firmware](../how-to/flash-ee02-standalone.md#buttons), which draws each page itself.

| `view=` | Page |
| --- | --- |
| (none) | Today's birds |
| `week` | This week's birds |
| `rarest` | The rarest bird lately |
| `story` | The day's story: dawn chorus, first and last singer, weather |
| `season` | Arrivals, this week last year, the year's list |

## Battery

GPIO6 switches on a divider that halves the battery's voltage onto GPIO1 (ADC1 channel 0, 11 dB
attenuation, curve calibration). Each wake takes nine readings, keeps the median, and doubles it.
A result under 2.5 V means no battery (USB power, or the switch off) and is not sent. GPIO6 is held
low through deep sleep, so the divider draws nothing.

## What the panel sends

```text
GET /panel?panel=13.3[&rotation=90][&view=week][&frame=hall] HTTP/1.1
Host: 192.168.1.20:8081
Accept-Encoding: deflate
If-None-Match: "b35332bb-z"          the page on the glass, if any
X-Frame-Battery-mV: 3912             the battery, if there is one
X-Frame-Firmware: 0.2.0              this firmware's version
Connection: close
```

The server may answer with `X-Frame-Firmware: <version>` on `200` and `304` answers, offering
the firmware it has at `GET /firmware/ee02-thin.bin`; see [Over-the-air updates](#over-the-air-updates).

## One wake

```mermaid
sequenceDiagram
    participant R as RTC memory
    participant F as ee02-thin
    participant W as Wi-Fi / server
    participant P as Panel
    F->>F: which button woke it, battery voltage, settings from flash
    F->>R: load SleepMemory (ETag, failures, Rejoin, view, update tries)
    alt Rejoin remembered
        F->>W: join that BSSID on its channel, reuse the address
    else none, or that failed
        F->>W: scan, join, DHCP
    end
    F->>W: GET /panel?panel=13.3&view=…, If-None-Match, battery, version
    W-->>F: 304, or 200 with the zlib page; maybe X-Frame-Firmware
    opt another version offered, battery ≥ 3.6 V, fewer than 3 failed tries
        F->>W: GET /firmware/ee02-thin.bin
    end
    Note over F: radio off
    opt 200, inflated to 960,000 bytes, CRC-32 matches the ETag
        F->>P: power on, init, send both halves, refresh (~30 s), power off
        Note over F,P: pins held low through sleep
    end
    opt firmware downloaded and checked
        F->>F: write the other app slot, select it, restart
    end
    F->>R: store SleepMemory
    Note over F: deep sleep for max-age (5 min to 12 h), or until a button
```

| Step | Limit |
| --- | --- |
| Join a remembered access point | 4 s, then a full scan |
| Full scan and join | 15 s |
| DHCP | 10 s |
| Connection stalled | 15 s |
| Response size | 1,000,000 bytes |
| Firmware size | one app slot, 3,932,160 bytes |
| Sleep | `max-age`, clamped to 5 min to 12 h; 15 min without one |
| After a failure | 5 min, doubled for each further failure in a row, up to 12 h |

What counts as a failure, and what is kept, follows `panel-wake`:

| Outcome | Glass | ETag kept | Rejoin kept | Sleep |
| --- | --- | --- | --- | --- |
| `304 Not Modified` | unchanged | same | yes | `max-age` |
| `200`, page checks out, panel refreshes | new page | the new one | yes | `max-age` |
| `200`, damaged, wrong length or not inflatable | unchanged | old | yes | back-off |
| `200`, panel fails | whatever the panel did | old, so it is fetched again | yes | back-off |
| Other status (e.g. `503` before the server's first page) | unchanged | old | yes | back-off |
| No Wi-Fi, no address, no connection | unchanged | old | no: next wake scans | back-off |

## Over-the-air updates

When an answer carries `X-Frame-Firmware` with a version other than the running one, the battery
reads at least 3.6 V (or there is none), and that version has not failed three times, the panel
downloads `/firmware/ee02-thin.bin` while the radio is still on. Before anything is written it
checks the download (`panel_wake::FirmwareImage`):

- the response is `200` with a `Content-Length`, and all of it arrived;
- the image header's magic byte and chip (ESP32-S3), every segment's length, and the XOR checksum;
- the application description: its project name is `ee02-thin` and its version is the one
  offered, so a mislabelled file cannot make the frame update on every wake.

It then writes the image to the app slot that is not running, reads every 4 KiB back, marks the
slot as the one to boot (`otadata`) and restarts. The bootloader checks the image's SHA-256
before it starts it, and falls back to the old slot if that fails. Once the new firmware reaches
the server it marks itself valid, so a bootloader built with rollback keeps it. A failed download
or install keeps the old firmware and counts a failed try; after three, that version is left alone
until the server offers another. Any other version counts, so the server can roll a frame back by
offering an older one.

## RTC memory

`panel_wake::SleepMemory`, 16 little-endian words in RTC fast memory (esp-hal's
`#[ram(unstable(rtc_fast, persistent))]`, as atomics so no `unsafe` is needed):

| Bytes | Content |
| --- | --- |
| 0 to 31 | The ETag of the page on the glass, quotes included |
| 32 | ETag length |
| 33 | Failed wakes in a row |
| 34 | 1 when a rejoin record follows |
| 35 to 40 | Access point BSSID |
| 41 | Channel |
| 42 to 45 | The panel's IPv4 address from the last DHCP lease |
| 46 | Prefix length |
| 47 | 1 when a gateway follows |
| 48 to 51 | Gateway |
| 52 | The view: 0 today, 1 week, 2 rarest, 3 story, 4 season |
| 53 to 56 | CRC-32 of the firmware version that failed to install |
| 57 | Failed tries at it |
| 58 and 59 | Zero |
| 60 to 63 | CRC-32 of bytes 0 to 59 |

Words that fail the CRC read as a first boot: no page known, no rejoin, today's view.

## Flash layout

`apps/ee02-thin/partitions.csv`, which `cargo run` passes to espflash. It fits the 8 MB and 16 MB
XIAO ESP32-S3 modules.

| Name | Type | Offset | Size | Holds |
| --- | --- | --- | --- | --- |
| `nvs` | data, nvs | `0x9000` | 20 KiB | Unused; kept for ESP-IDF tools |
| `otadata` | data, ota | `0xe000` | 8 KiB | Which app slot boots, and its state |
| `ota_0` | app | `0x10000` | 3.75 MiB | The firmware |
| `ota_1` | app | `0x3d0000` | 3.75 MiB | The other firmware, for updates |
| `frame` | data, undefined | `0x790000` | 4 KiB | The settings record |

Flashing over USB erases `otadata` (`--erase-parts otadata`), so the board boots what was just
flashed into `ota_0` rather than an older update in `ota_1`. It leaves `frame` alone, so the
settings survive.

## Pins

The panel's, from Seeed's `XIAO_ePaper_Board_Configs.h` by way of fugleramme's EE02 firmware;
the buttons and battery are in [Buttons](#buttons) and [Battery](#battery).

| Signal | GPIO | Notes |
| --- | --- | --- |
| SPI clock | 7 | SPI2 at 10 MHz, mode 0 |
| SPI data out | 9 | |
| Chip select, first controller | 44 | Also UART0 RX |
| Chip select, second controller | 41 | |
| Data/command | 10 | |
| Reset | 38 | |
| BUSY | 4 | Input; low while the controllers work |
| Panel power | 43 | Load switch, high for power. Also UART0 TX. |

Because GPIO43 and GPIO44 are UART0's pins, the firmware logs over the ESP32-S3's built-in USB
serial port only (`esp-println`'s `jtag-serial`). After a refresh the panel is powered off and
clock, data, both chip selects, data/command and reset are held low through deep sleep, so the
idle panel is not fed through its inputs; the next wake that draws releases the holds.

## The reTerminal E1002

Built with `--no-default-features --features board-e1002`, the same firmware runs on Seeed's
reTerminal E1002 (ESP32-S3, 8 MB octal PSRAM, 32 MB flash) and its 7.3" panel: it asks for
`panel=7.3` and takes a 192,000-byte page. Each board's table is in `apps/ee02-thin/src/board.rs`;
the panel code is `crates/glass-esp`, shared with the [standalone firmware](../how-to/flash-ee02-standalone.md).

| Signal | GPIO | Notes |
| --- | --- | --- |
| SPI clock, data out | 7, 9 | SPI2 at 4 MHz, mode 0, as Zephyr's board files |
| Chip select, data/command, reset | 10, 11, 12 | Held high, low and high through deep sleep |
| BUSY | 13 | Input, pulled up; low while the controller works |
| Refresh, previous, next buttons | 3, 5, 4 | The green button, the left and the right one |
| Battery divider, its switch | 1, 21 | As on the EE02, with another switch pin |

The panel has no power switch: the controller's own deep sleep keeps it quiet. The button and
battery pins are from Seeed's reTerminal E10xx documentation and, like the rest, untried.

Its application description names it `e1002-thin`, so it never installs the EE02's update.
The server serves one firmware file to every frame, so an E1002 offered the EE02's firmware
refuses it, and after three refusals leaves that version alone; update E1002s over USB, or point
them at a server that serves the E1002's build.

## Toolchain and crates

| | |
| --- | --- |
| Target | `xtensa-esp32s3-none-elf`, `no_std`, `build-std = ["alloc", "core"]` |
| Toolchain | Espressif's Rust 1.98.1.0 from `espup` (named `esp`) |
| HAL | `esp-hal` 1.2, `esp-rtos` with embassy, `esp-radio` 1.0 beta for Wi-Fi, `embassy-net` for TCP/IP |
| Heap | 72 KiB of internal RAM reclaimed from the bootloader, plus the 8 MB PSRAM for the page |
| Image | about 580 KB; flashes into `ota_0` at `0x10000` behind the ESP-IDF bootloader |

The firmware has its own Cargo workspace and `Cargo.lock` in `apps/ee02-thin`; the root workspace
excludes it and `crates/transport-esp`, so `cargo test --workspace` never needs the Xtensa
toolchain. See [Dependencies](../explanation/dependencies.md#the-firmware) for what it links.
