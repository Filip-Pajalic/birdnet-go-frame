# HTTP endpoints

All endpoints are `GET`, need no authentication, and answer `503` until the first page is drawn.

| Path | Content type | Body |
| --- | --- | --- |
| `/` | `text/html` | A page showing `/preview.png` and links to the others |
| `/frame.e6` | `application/octet-stream` | The page for a panel, see [E6F1](e6-format.md) |
| `/frame.png` | `image/png` | 8-bit palette PNG, six nominal ink colours: a panel's native landscape, or a canvas as seen |
| `/preview.png` | `image/png` | RGB PNG before the panel boost and dithering, as the page hangs |
| `/healthz` | `text/plain` | `ok`, or `503` before the first page |

## Sizes

Every client can say what it is, so one server feeds panels and screens of different sizes. The
three page paths take:

| Query | Means | Example |
| --- | --- | --- |
| none | The configured `PANEL` at the configured `ROTATION` | `/frame.e6` |
| `panel=7.3` or `panel=13.3`, `rotation=0\|90\|180\|270` | An e-ink panel; either may be left out and comes from the settings | `/frame.e6?panel=7.3&rotation=90` |
| `w` and `h` | A canvas of any size, 64 to 3840 pixels a side and at most 3840x2160 in all; PNG only | `/preview.png?w=1920&h=1080` |

| Panel | Page as it hangs | `/frame.e6` payload |
| --- | --- | --- |
| 13.3" (1600x1200) | 1600x1200, or 1200x1600 in portrait | Quarter turn to the controllers' 1200x1600, each row split between two controllers (fugleramme's layout) |
| 7.3" (800x480) | 800x480, or 480x800 in portrait | Native landscape rows, 800 wide, in order, for one controller |

The header's width and height say which layout a page holds. A value that cannot be read, a
canvas outside the limits, `w` without `h`, or `/frame.e6` with `w` and `h` gets `400` with the
reason in the body.

Each size is drawn the first time it is asked for after the birds change (under a second for a
collage), then kept, so later requests and 304s are instant. At most eight sizes are kept at
once.

## Caching

`/frame.e6`, `/frame.png` and `/preview.png` carry `Cache-Control: no-cache` and, for each size,
one shared `ETag`, derived from the E6F1 bytes (or a canvas's six-colour PNG). A request with a matching `If-None-Match` gets `304 Not Modified` and
no body. The ETag changes only when the page is redrawn, which happens when the set of species
changes.

## BirdNET-Go endpoints the server calls

| Path | When |
| --- | --- |
| `GET /api/v2/analytics/species/summary?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` | Every `REFRESH_SECONDS` |

`birdnet-client` also implements `GET /api/v2/detections/recent?limit=N` for later layouts.
