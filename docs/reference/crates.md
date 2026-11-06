# Crates and apps

| Crate | `no_std` | Purpose | Key items |
| --- | --- | --- | --- |
| `frame-core` | yes | Shared page format | `Ink`, `Rotation`, `PanelSize`, `IndexedImage`, `E6Header`, `Crc32`, `e6::encode` |
| `birdnet-model` | yes (alloc) | BirdNET-Go response types | `SpeciesSummary`, `Detection`, `ScientificName` |
| `birdnet-client` | yes (alloc) | API client | `BirdnetClient<T>`, `HttpTransport`, `DateWindow`, `CivilDate` |
| `birdnet-mock` | yes (alloc) | Simulated station | `MockBirdnet`, `MockTransport` |
| `transport-ureq` | no | `HttpTransport` for Linux | `UreqTransport` |
| `frame-layout` | yes (alloc) | Page layout as data | `PageLayout`, `TextBlock`, `TextList` |
| `frame-raster` | yes (alloc) | Drawing and dithering | `Renderer`, `Canvas`, `dither` |
| `frame-http` | no | HTTP endpoints | `FrameServer`, `PageStore`, `RenderedPage` |

| App | Binary | Uses |
| --- | --- | --- |
| `apps/server` | `birdnet-go-frame` | client + (ureq or mock) + layout + raster + http |
| `apps/fake-birdnet` | `fake-birdnet` | mock over HTTP |

## Planned

| Crate or app | Purpose |
| --- | --- |
| `panel-t133a01` | 13.3" Spectra 6 driver on `embedded-hal` 1.0 traits |
| `transport-esp` | `HttpTransport` and Wi-Fi on `esp-idf-svc` |
| `art-pack` | Pre-decoded art format a microcontroller can read from SD |
| `apps/ee02-thin` | ESP32 firmware that fetches `/frame.e6` |
| `apps/ee02-standalone` | ESP32 firmware that renders the page itself |
| `apps/linux-panel` | Raspberry Pi that renders and drives the panel |
| `tools/art-packer` | Converts fugleramme's webp art to `art-pack` |
