# Crates and apps

| Crate | `no_std` | Purpose | Key items |
| --- | --- | --- | --- |
| `frame-core` | yes | Shared page format, and unpacking it | `Ink`, `Rotation`, `PanelSize`, `Glass` (what every panel driver does), `IndexedImage`, `PageSizeError`, `Crc32`, `panel_bytes::pack`, `inflate_zlib` |
| `lite-json` | yes (alloc) | A small JSON reader, in place of serde | `Json`, `JsonError` |
| `birdnet-model` | yes (alloc) | BirdNET-Go response types | `SpeciesSummary`, `SpeciesHours`, `Detection`, `ScientificName` |
| `birdnet-client` | yes (alloc) | API client | `BirdnetClient<T>`, `HttpTransport`, `DateWindow`, `CivilDate` |
| `birdnet-mock` | yes (alloc) | Simulated station, with seasons, rarities and weather | `MockBirdnet`, `MockTransport` |
| `lan-http` | no | Plain HTTP/1.1 GET client and small server | `get`, `Server`, `Request`, `Response` |
| `transport-lan` | no | `HttpTransport` for Linux, macOS, Windows | `LanTransport` |
| `art-pack` | yes (alloc) | Bird cut-outs, their `.artp` format, bundles of them for flash, no art, and plain ovals for tests | `ArtImage`, `ArtSource`, `ArtBundle`, `NoArt`, `OutlineArt`, `Rgba`, `Subject` |
| `art-webp` | no | fugleramme's art folders: plate per day, paper halo toned and feathered; the border's plants, built in | `ArtFolder`, `BorderSprays` |
| `bird-sizes` | yes | AVONET body masses for BirdNET's species, and the name-keyed table they and the facts are read with | `BirdSizes`, `NameTable` |
| `bird-facts` | yes | A few facts per species (AVONET and Wikidata, built in), one told a day about a page's main bird | `BirdFacts`, `Fact` |
| `page-text` | yes (alloc) | The page's words in a dozen languages, from a translations JSON | `Translations`, `PageText` |
| `frame-layout` | yes (alloc) | Page layout as data; the collage's spiral packing by outline, round the border's plants; which border a date wears | `PageLayout`, `Collage`, `Silhouette`, `TextList`, `StoryPage`, `SeasonPage`, `CornerSprays`, `Spray`, `BorderTheme`, `Border`, `ImageBlock`, `ShapeBlock`, `TextBlock`, `size_weights` |
| `frame-raster` | yes (alloc) | Drawing, the panel boost, and dithering | `Renderer`, `Canvas`, `pop`, `dither` |
| `frame-artist` | yes (alloc) | Draws the page for a list of birds: layout choice, pictures, sizes, a featured bird, the story and season pages, which page each view shows, the fact line and the border, and the panel's bytes. Shared by the server and the apps that draw their own page | `Artist`, `PageStyle`, `Layout`, `PageView`, `ViewData`, `DayFacts`, `YearFacts`, `fun_fact`, `panel_bytes` |
| `bird-almanac` | yes (alloc) | What longer windows say: birds new for the year, arrivals after an absence, the rarest of today's birds, this week against last year's, and the day's story | `Almanac`, `Arrival`, `YearOnYear`, `DayStory` |
| `open-meteo` | yes (alloc) | Today's weather from Open-Meteo's free forecast API, over any `HttpTransport` | `OpenMeteo`, `Weather`, `Sky` |
| `home-assistant` | no | Announces frames to Home Assistant: just enough MQTT 3.1.1 to publish, and the discovery messages | `mqtt::publish`, `discovery::frame_messages` |
| `frame-standalone` | yes (alloc) | The decisions of a frame that draws its own page: whether the birds changed, the request to BirdNET-Go and reading its answer and `Date`, what each page asks for and makes of the answers, what to keep in RTC memory (the page the buttons chose included) and how long to sleep | `Fingerprint`, `birdnet_request`, `BirdnetAnswer`, `http_date::parse`, `ViewFetch`, `StandaloneMemory` |
| `frame-http` | no | HTTP endpoints, PNG and zlib encoding | `FrameServer`, `PageStore`, `RenderedPage`, `PanelBody`, `Target`, `FrameLog`, `query_value` |
| `panel-t133a01` | yes | The 13.3" Spectra 6 panel's two controllers on `embedded-hal` 1.0 | `T133a01`, `Pins`, `PanelError` |
| `panel-ed2208` | yes | The 7.3" Spectra 6 panel's ED2208 controller on `embedded-hal` 1.0 (reTerminal E1002, Waveshare 7.3" (E)) | `Ed2208`, `Pins`, `PanelError` |
| `panel-wake` | yes (alloc) | A panel's wake: the request (view, frame name, battery, version), reading the answer, unpacking and checking the page, planning the sleep, button presses, whether and what firmware to install, what to keep in RTC memory | `WakeState`, `ResponseHead`, `Wake`, `SleepLimits`, `SleepMemory`, `Rejoin`, `FrameAddress`, `PageRequest`, `PageView`, `Button`, `FirmwareImage`, `FirmwareUpdate`, `battery_millivolts`, `page_request` |
| `panel-setup` | yes (alloc) | A panel's setup hotspot: the settings record kept in flash, the setup page and its form, and just enough DHCP and DNS for a phone to find the page | `FrameSettings`, `SettingsError`, `SetupRequest`, `DhcpServer`, `dns::captive_answer`, `setup_page::render` |
| `glass-esp` | yes | Each ESP32-S3 board's panel: its pins, power order and pin holds through deep sleep, over esp-hal, handing the bus to the panel drivers. Built only with the Xtensa toolchain, outside the root workspace | `ee02::show`, `e1002::show`, `GlassPins`, `Refresh` |
| `transport-esp` | yes (alloc) | ESP32 Wi-Fi with fast rejoin and one HTTP GET; the setup hotspot and serving UDP and HTTP on it; on `esp-radio` and `embassy-net`. Built only with the Xtensa toolchain, outside the root workspace | `join`, `network_config`, `wait_for_address`, `get`, `rejoin`, `scan`, `open_hotspot`, `serve_udp`, `serve_http`, `WifiCredentials`, `TransportError` |

| App | Binary | Uses |
| --- | --- | --- |
| `apps/server` | `birdnet-go-frame` | client + (transport-lan or mock) + (art-webp or none) + page-text + almanac + open-meteo + artist + raster + http + home-assistant |
| `apps/fake-birdnet` | `fake-birdnet` | mock + lan-http |
| `apps/linux-panel` | `birdnet-go-frame-panel` (Raspberry Pi) | client + (transport-lan or mock) + (art-webp or none) + page-text + artist + standalone + panel-t133a01 / panel-ed2208 on linux-embedded-hal |
| `tools/art-packer` | `art-packer` | art-webp + art-pack + frame-http's zlib: packs plates, and the border's plants, into a bundle for firmware flash |
| `apps/ee02-thin` | `ee02-thin` (ESP32-S3 firmware, EE02 or reTerminal E1002) | panel-wake + panel-setup + transport-esp + glass-esp + frame-core, on esp-hal; its own workspace |
| `apps/ee02-standalone` | `ee02-standalone` (ESP32-S3 firmware with no server, EE02 or reTerminal E1002) | birdnet-client + frame-standalone + artist + raster + art-pack (a bundle in flash, or none) + page-text + transport-esp + glass-esp, on esp-hal; its own workspace |

## Planned

| Crate or app | Purpose |
| --- | --- |
| `apps/ee02-standalone` | ESP32 firmware that renders the page itself |
