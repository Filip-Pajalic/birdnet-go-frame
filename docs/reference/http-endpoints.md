# HTTP endpoints

All endpoints are `GET`, need no authentication, and answer `503` until the first page is drawn.

| Path | Content type | Body |
| --- | --- | --- |
| `/` | `text/html` | A page showing `/preview.png` and links to the others |
| `/frame.e6` | `application/octet-stream` | The page for the 13.3" panel, see [E6F1](e6-format.md) |
| `/frame.png` | `image/png` | 8-bit palette PNG, six nominal ink colours, native landscape (1600x1200) |
| `/preview.png` | `image/png` | RGB PNG before dithering, as the page hangs |
| `/healthz` | `text/plain` | `ok`, or `503` before the first page |

## Caching

`/frame.e6`, `/frame.png` and `/preview.png` carry `Cache-Control: no-cache` and one shared `ETag`,
derived from the E6F1 bytes. A request with a matching `If-None-Match` gets `304 Not Modified` and
no body. The ETag changes only when the page is redrawn, which happens when the set of species
changes.

## BirdNET-Go endpoints the server calls

| Path | When |
| --- | --- |
| `GET /api/v2/analytics/species/summary?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` | Every `REFRESH_SECONDS` |

`birdnet-client` also implements `GET /api/v2/detections/recent?limit=N` for later layouts.
