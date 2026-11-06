# HTTP endpoints

All endpoints are `GET`, need no authentication, and answer `503` until the first page is drawn.

| Path | Content type | Body |
| --- | --- | --- |
| `/` | `text/html` | A page showing `/preview.png` and links to the others |
| `/panel` | `application/octet-stream` | A panel's controller bytes, zlib-compressed when the client sends `Accept-Encoding: deflate`; see [The panel format](panel-format.md) |
| `/frame.png` | `image/png` | 8-bit palette PNG, six nominal ink colours: a panel's native landscape, or a canvas as seen |
| `/preview.png` | `image/png` | RGB PNG before the panel boost and dithering, as the page hangs |
| `/healthz` | `text/plain` | `ok`, or `503` before the first page |
| `/api/frames` | `application/json` | Every frame's last wake, battery and firmware; see [Frames' wakes](#frames-wakes). Answers before the first page too |
| `/api/frames/<name>` | `application/json` | One frame's, or `404` until it has woken |
| `/firmware/<name>.bin` | `application/octet-stream` | A file from `FIRMWARE_DIR`, with `Content-Length`; `404` when unset or missing |

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

## Frames and views

The page paths also take:

| Query | Means | Example |
| --- | --- | --- |
| `frame=<name>` | A frame from `FRAMES`: its layout, panel, rotation and default view. An unknown or missing name gets the server's own settings. | `/panel?frame=desk` |
| `view=today\|week\|rarest\|story\|season` | Which page; see [Pages and views](../explanation/pages-and-views.md). Missing or unknown values get the frame's default (`VIEW` unless the frame sets one), never a 400. | `/panel?frame=hall&view=story` |

`panel=` and `rotation=` still override the frame's panel and rotation. The EE02 firmware sends
`frame=` from its setup page and cycles `view=` with its button.

A value that cannot be read, a canvas outside the limits, `w` without `h`, or `/panel` with `w`
and `h` gets `400` with the reason in the body.

Each size is drawn the first time it is asked for after the birds change (under a second for a
collage), then kept, so later requests and 304s are instant. At most 24 pages, sizes and views
together, are kept at once.

## Caching

Every page path carries an `ETag`, derived from the panel's bytes (or a canvas's six-colour PNG),
so all of a size's paths change together. A request with a matching `If-None-Match` gets
`304 Not Modified` and no body. The ETag changes only when the page is redrawn, which happens when
the set of species changes. The compressed `/panel` has its own ETag, the same with `-z` added,
as HTTP asks of the encodings of one resource.

`/frame.png` and `/preview.png` carry `Cache-Control: no-cache`, so browsers ask again every time.
`/panel` carries `Cache-Control: max-age=<POLL_MINUTES x 60>`: the time the device should sleep
before asking again. While a bird new for the year has been heard within `ALERT_MINUTES`, it is
`ALERT_POLL_MINUTES x 60` instead (5 minutes unless changed), so frames wake early to show it.
`QUIET_HOURS` still wins at night.

## Frames' wakes

`/panel` reads two request headers and notes them, with the time, under the frame's name:

| Header | Example | Means |
| --- | --- | --- |
| `X-Frame-Battery-mV` | `3912` | The battery in millivolts; left out on USB power |
| `X-Frame-Firmware` | `0.2.0` | The firmware's version |

`/api/frames` lists what each frame said last:

```json
[{"frame":"hall","last_wake":1793962947,"battery_mv":3870,"battery_percent":65,"firmware":"0.2.0","view":"","wakes":12}]
```

`last_wake` is unix seconds. `battery_percent` is read off a typical lithium-ion discharge curve
(3.3 V empty, 4.15 V full) and is rough. The log lives in memory: it starts empty when the
server restarts, and keeps the 32 frames heard from most recently.

## Firmware

With `FIRMWARE_DIR` set and both `ee02-thin.bin` and `ee02-thin.version` in it, every `/panel`
answer (200 and 304) carries `X-Frame-Firmware: <version>`, the trimmed contents of the version
file. A frame on another version fetches `/firmware/ee02-thin.bin` and installs it. File names
under `/firmware/` are `a-z`, `0-9`, `-`, `.` and `_`, ending in `.bin`.

`/frame.e6`, fugleramme's format, is gone: see
[Why not fugleramme's `.e6`](panel-format.md#why-not-fuglerammes-e6).

## BirdNET-Go endpoints the server calls

| Path | When |
| --- | --- |
| `GET /api/v2/analytics/species/summary?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD` | Every `REFRESH_SECONDS` for the `WINDOW_HOURS` window; every `ALMANAC_MINUTES`, and when today's birds change, for today, the last 7 days, the year so far, the same 7 days a year earlier, the last 21 days and the 60 before them |
| `GET /api/v2/analytics/time/hourly/batch?date=YYYY-MM-DD&species=...` | With the above, for today's 8 most heard birds and any new for the year (10 a request) |

The weather comes from `GET <WEATHER_URL>/v1/forecast?latitude=..&longitude=..&current=..&daily=..`
when `LATITUDE` and `LONGITUDE` are set.

`birdnet-client` also implements `GET /api/v2/detections/recent?limit=N` for later layouts.
