# Crates and apps

| Crate | `no_std` | Purpose | Key items |
| --- | --- | --- | --- |
| `frame-core` | yes | Shared page format, and unpacking it | `Ink`, `Rotation`, `PanelSize`, `Glass` (what every panel driver does), `IndexedImage`, `PageSizeError`, `Crc32`, `panel_bytes::pack`, `inflate_zlib` |
| `lite-json` | yes (alloc) | A small JSON reader, in place of serde | `Json`, `JsonError` |
| `birdnet-model` | yes (alloc) | BirdNET-Go response types | `SpeciesSummary`, `Detection`, `ScientificName` |
| `birdnet-client` | yes (alloc) | API client | `BirdnetClient<T>`, `HttpTransport`, `DateWindow`, `CivilDate` |
| `birdnet-mock` | yes (alloc) | Simulated station | `MockBirdnet`, `MockTransport` |
| `lan-http` | no | Plain HTTP/1.1 GET client and small server | `get`, `Server`, `Request`, `Response` |
| `transport-lan` | no | `HttpTransport` for Linux, macOS, Windows | `LanTransport` |
| `art-pack` | yes (alloc) | Bird cut-outs and their `.artp` format | `ArtImage`, `ArtSource`, `Rgba`, `Subject` |
| `art-mock` | yes (alloc) | Generated bird pictures for any species | `MockArt` |
| `art-webp` | no | fugleramme's art folders: plate per day, paper halo toned and feathered | `ArtFolder` |
| `bird-sizes` | yes | AVONET body masses for BirdNET's species | `BirdSizes` |
| `page-text` | yes (alloc) | The page's words in a dozen languages, from a translations JSON | `Translations`, `PageText` |
| `frame-layout` | yes (alloc) | Page layout as data; the collage's spiral packing by outline | `PageLayout`, `Collage`, `Silhouette`, `TextList`, `ImageBlock`, `TextBlock`, `size_weights` |
| `frame-raster` | yes (alloc) | Drawing, the panel boost, and dithering | `Renderer`, `Canvas`, `pop`, `dither` |
| `frame-artist` | yes (alloc) | Draws the page for a list of birds: layout choice, pictures, sizes, and the panel's bytes. Shared by the server and the apps that draw their own page | `Artist`, `PageStyle`, `Layout`, `panel_bytes` |
| `frame-http` | no | HTTP endpoints, PNG and zlib encoding | `FrameServer`, `PageStore`, `RenderedPage`, `PanelBody`, `Target` |
| `panel-t133a01` | yes | The 13.3" Spectra 6 panel's two controllers on `embedded-hal` 1.0 | `T133a01`, `Pins`, `PanelError` |
| `panel-ed2208` | yes | The 7.3" Spectra 6 panel's ED2208 controller on `embedded-hal` 1.0 (reTerminal E1002, Waveshare 7.3" (E)) | `Ed2208`, `Pins`, `PanelError` |
| `panel-wake` | yes (alloc) | A panel's wake: the request, reading the answer, unpacking and checking the page, planning the sleep, what to keep in RTC memory | `WakeState`, `ResponseHead`, `Wake`, `SleepLimits`, `SleepMemory`, `Rejoin`, `FrameAddress`, `page_request` |
| `transport-esp` | yes (alloc) | ESP32 Wi-Fi with fast rejoin and one HTTP GET, on `esp-radio` and `embassy-net`. Built only with the Xtensa toolchain, outside the root workspace | `join`, `network_config`, `wait_for_address`, `get`, `rejoin`, `WifiCredentials`, `TransportError` |

| App | Binary | Uses |
| --- | --- | --- |
| `apps/server` | `birdnet-go-frame` | client + (transport-lan or mock) + (art-webp or art-mock) + page-text + artist + raster + http |
| `apps/fake-birdnet` | `fake-birdnet` | mock + lan-http |
| `apps/ee02-thin` | `ee02-thin` (ESP32-S3 firmware) | panel-wake + transport-esp + panel-t133a01 + frame-core, on esp-hal; its own workspace |

## Planned

| Crate or app | Purpose |
| --- | --- |
| `apps/ee02-standalone` | ESP32 firmware that renders the page itself |
| `apps/linux-panel` | Raspberry Pi that renders and drives the panel |
| `tools/art-packer` | Converts fugleramme's webp art to `.artp`, so the ESP32 can read plates from an SD card |
