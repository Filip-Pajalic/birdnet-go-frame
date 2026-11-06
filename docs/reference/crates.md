# Crates and apps

| Crate | `no_std` | Purpose | Key items |
| --- | --- | --- | --- |
| `frame-core` | yes | Shared page format | `Ink`, `Rotation`, `PanelSize`, `IndexedImage`, `E6Header`, `Crc32`, `e6::encode` |
| `birdnet-model` | yes (alloc) | BirdNET-Go response types, with a built-in JSON reader | `SpeciesSummary`, `Detection`, `ScientificName` |
| `birdnet-client` | yes (alloc) | API client | `BirdnetClient<T>`, `HttpTransport`, `DateWindow`, `CivilDate` |
| `birdnet-mock` | yes (alloc) | Simulated station | `MockBirdnet`, `MockTransport` |
| `lan-http` | no | Plain HTTP/1.1 GET client and small server | `get`, `Server`, `Request`, `Response` |
| `transport-lan` | no | `HttpTransport` for Linux, macOS, Windows | `LanTransport` |
| `art-pack` | yes (alloc) | Bird cut-outs and their `.artp` format | `ArtImage`, `ArtSource`, `Rgba` |
| `art-mock` | yes (alloc) | Generated bird pictures for any species | `MockArt` |
| `bird-sizes` | yes | AVONET body masses for BirdNET's species | `BirdSizes` |
| `frame-layout` | yes (alloc) | Page layout as data | `PageLayout`, `Collage`, `TextList`, `ImageBlock`, `TextBlock`, `size_weights` |
| `frame-raster` | yes (alloc) | Drawing and dithering | `Renderer`, `Canvas`, `dither` |
| `frame-http` | no | HTTP endpoints and PNG encoding | `FrameServer`, `PageStore`, `RenderedPage` |

| App | Binary | Uses |
| --- | --- | --- |
| `apps/server` | `birdnet-go-frame` | client + (transport-lan or mock) + art-mock + bird-sizes + layout + raster + http |
| `apps/fake-birdnet` | `fake-birdnet` | mock + lan-http |

## Planned

| Crate or app | Purpose |
| --- | --- |
| `panel-t133a01` | 13.3" Spectra 6 driver on `embedded-hal` 1.0 traits |
| `transport-esp` | `HttpTransport` and Wi-Fi on `esp-idf-svc` |
| `apps/ee02-thin` | ESP32 firmware that fetches `/frame.e6` |
| `apps/ee02-standalone` | ESP32 firmware that renders the page itself |
| `apps/linux-panel` | Raspberry Pi that renders and drives the panel |
| `tools/art-packer` | Converts fugleramme's webp art to `.artp`, for an `ArtSource` that reads real art |
