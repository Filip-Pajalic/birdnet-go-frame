# HTTP endpoints

All endpoints are `GET`, need no authentication, and answer `503` until the first page is drawn.

| Path | Content type | Body |
| --- | --- | --- |
| `/` | `text/html` | A page showing `/preview.png` and links to the others |
| `/panel` | `application/octet-stream` | A panel's controller bytes, zlib-compressed when the client sends `Accept-Encoding: deflate`; see [The panel format](panel-format.md) |
| `/frame.png` | `image/png` | 8-bit palette PNG, six nominal ink colours: a panel's native landscape, or a canvas as seen |
| `/preview.png` | `image/png` | RGB PNG before the panel boost and dithering, as the page hangs |
| `/healthz` | `text/plain` | `ok`, or `503` before the first page |

## Sizes

Every client can say what it is, so one server feeds panels and screens of different sizes. The
three page paths take:

| Query | Means | Example |
| --- | --- | --- |
| none | The configured `PANEL` at the configured `ROTATION` | `/panel` |
| `panel=7.3` or `panel=13.3`, `rotation=0\|90\|180\|270` | An e-ink panel; either may be left out and comes from the settings | `/panel?panel=7.3&rotation=90` |
| `w` and `h` | A canvas of any size, 64 to 3840 pixels a side and at most 3840x2160 in all; PNG only | `/preview.png?w=1920&h=1080` |

| Panel | Page as it hangs | `/panel` bytes |
| --- | --- | --- |
| 13.3" (1600x1200) | 1600x1200, or 1200x1600 in portrait | Quarter turn to the controllers' 1200x1600, each row split between two controllers (fugleramme's layout) |
| 7.3" (800x480) | 800x480, or 480x800 in portrait | Native landscape rows, 800 wide, in order, for one controller |

A value that cannot be read, a canvas outside the limits, `w` without `h`, or `/panel` with `w`
and `h` gets `400` with the reason in the body.

Each size is drawn the first time it is asked for after the birds change (under a second for a
collage), then kept, so later requests and 304s are instant. At most eight sizes are kept at
once.

## Caching

Every page path carries an `ETag`, derived from the panel's bytes (or a canvas's six-colour PNG),
so all of a size's paths change together. A request with a matching `If-None-Match` gets
`304 Not Modified` and no body. The ETag changes only when the page is redrawn, which happens when
the set of species changes. The compressed `/panel` has its own ETag, the same with `-z` added,
as HTTP asks of the encodings of one resource.

`/frame.png` and `/preview.png` carry `Cache-Control: no-cache`, so browsers ask again every time.
`/panel` carries `Cache-Control: max-age=<POLL_MINUTES x 60>`: the time the device should sleep
before asking again.

`/frame.e6`, fugleramme's format, is gone: see
[Why not fugleramme's `.e6`](panel-format.md#why-not-fuglerammes-e6).

## BirdNET-Go endpoints the server calls

| Path | When |
| --- | --- |
| `GET /api/v2/analytics/species/summary?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` | Every `REFRESH_SECONDS` |

`birdnet-client` also implements `GET /api/v2/detections/recent?limit=N` for later layouts.
