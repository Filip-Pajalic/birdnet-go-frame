# Crates and apps

| Crate | `no_std` | Purpose | Key items |
| --- | --- | --- | --- |
| `frame-core` | yes | Shared page format | `Ink`, `Rotation`, `PanelSize`, `IndexedImage`, `PageSizeError`, `Crc32`, `panel_bytes::pack` |
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
| `frame-http` | no | HTTP endpoints, PNG and zlib encoding | `FrameServer`, `PageStore`, `RenderedPage`, `PanelBody`, `Target` |

| App | Binary | Uses |
| --- | --- | --- |
| `apps/server` | `birdnet-go-frame` | client + (transport-lan or mock) + (art-webp or art-mock) + bird-sizes + page-text + layout + raster + http |
| `apps/fake-birdnet` | `fake-birdnet` | mock + lan-http |

## Planned

| Crate or app | Purpose |
| --- | --- |
| `panel-t133a01` | 13.3" Spectra 6 driver on `embedded-hal` 1.0 traits |
| `transport-esp` | `HttpTransport` and Wi-Fi on `esp-idf-svc` |
| `apps/ee02-thin` | ESP32 firmware that fetches `/panel` |
| `apps/ee02-standalone` | ESP32 firmware that renders the page itself |
| `apps/linux-panel` | Raspberry Pi that renders and drives the panel |
| `tools/art-packer` | Converts fugleramme's webp art to `.artp`, so the ESP32 can read plates from an SD card |
