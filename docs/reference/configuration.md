# Configuration

The server reads environment variables at start-up.

| Variable | Default | Type | Meaning |
| --- | --- | --- | --- |
| `BIRDNET_URL` | required | URL or `mock` | BirdNET-Go base URL, e.g. `http://192.168.1.10:8080`. `mock` uses the built-in simulated station. |
| `LISTEN` | `0.0.0.0:8080` | address | Where the server listens. |
| `ROTATION` | `0` | `0`, `90`, `180`, `270` | How the panel hangs, counter-clockwise from native landscape. `90` and `270` are portrait (1200x1600). |
| `POLL_MINUTES` | `15` | 0 to 255 | Minutes the panel sleeps between checks, sent in the page header. `0` means as soon as the panel likes. |
| `WINDOW_HOURS` | `24` | hours | How far back detections count. Rounded outwards to whole days, as BirdNET-Go's summary takes dates. |
| `UTC_OFFSET_MINUTES` | `0` | minutes | The garden's time zone, minutes east of UTC. Used for the date range and the "Updated" time. Not adjusted for daylight saving. |
| `REFRESH_SECONDS` | `60` | seconds | How often the server asks BirdNET-Go for news. |
| `ART_DIR` | unset | folder | A fugleramme art style folder (`birds/*.webp`, `geometry.json`), e.g. from `scripts/fetch-art.sh`. Unset draws generated birds. Species without a plate are left off the collage. |
| `LAYOUT` | `collage` | `collage` or `list` | `collage` packs a picture of every bird by its outline, heavier birds bigger. `list` writes names and counts only. |
| `MAX_BIRDS` | `0` | number | Most birds on a page, most heard first. `0` shows every bird heard, as fugleramme does. |
| `NAMES` | `true` | `true` or `false` | Whether the collage writes each bird's name under it. |
| `HEADING` | unset | text | A heading over the page. Unset: none on the collage (birds only, like fugleramme), the translated "Heard in the garden" on the list. |
| `LANGUAGE` | `en` | language code | Language of the page's own words (heading, "No birds heard yet", "Updated 07:30"), e.g. `sv`, `nb`, `de`, `pt-BR`. A regional code falls back to its main language, then to English. Bird names come from BirdNET-Go in its own language setting. |
| `TRANSLATIONS` | unset | file | A translations JSON adding languages or overriding words; see below. |

## Translations

The bundled file, `crates/page-text/data/translations.json`, covers English, Danish, Dutch,
Finnish, French, German, Italian, Norwegian Bokmål, Polish, Portuguese, Spanish and Swedish. A
file named by `TRANSLATIONS` has the same shape and wins word by word, so it can be partial:

```json
{
  "sv": { "heading": "Hört vid fågelbordet" },
  "is": {
    "heading": "Heyrt í garðinum",
    "nothing_heard": "Engir fuglar heyrst enn",
    "more": "og {count} til viðbótar",
    "updated": "Uppfært {time}",
    "unreachable": "Næ ekki sambandi við BirdNET-Go"
  }
}
```

| Key | Where | Placeholder |
| --- | --- | --- |
| `heading` | Heading of the list page | |
| `nothing_heard` | Shown when no bird has been heard | |
| `more` | Counts species left off the list | `{count}` |
| `updated` | Foot of the list page | `{time}` |
| `unreachable` | Heading when BirdNET-Go cannot be reached | |

An invalid value stops the server with a message naming the variable, e.g.
`ROTATION: cannot read "45": rotation must be 0, 90, 180 or 270`.

## `fake-birdnet`

| Setting | Default | Meaning |
| --- | --- | --- |
| first argument | `127.0.0.1:8090` | Address to listen on |
| `UTC_OFFSET_MINUTES` | `0` | Time zone of the simulated garden |
