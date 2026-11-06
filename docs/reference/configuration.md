# Configuration

The server reads environment variables at start-up.

| Variable | Default | Type | Meaning |
| --- | --- | --- | --- |
| `BIRDNET_URL` | required | URL or `mock` | BirdNET-Go base URL, e.g. `http://192.168.1.10:8080`. `mock` uses the built-in simulated station. |
| `BIRDNET_PASSWORD` | unset | text | BirdNET-Go's password, when its private mode is on; the server logs in with it. See [If BirdNET-Go is private](../how-to/connect-birdnet-go.md#if-birdnet-go-is-private). Never printed. |
| `BIRDNET_USERNAME` | `admin` | text | The name sent with `BIRDNET_PASSWORD`; BirdNET-Go's password login takes any. |
| `LISTEN` | `0.0.0.0:8080` | address | Where the server listens. |
| `PANEL` | `13.3` | `13.3` or `7.3` | The panel served to clients that do not name one. Clients can ask for another panel or any screen size per request; see [HTTP endpoints](http-endpoints.md#sizes). |
| `ROTATION` | `0` | `0`, `90`, `180`, `270` | How the panel hangs, counter-clockwise from native landscape, for clients that do not say. `90` and `270` are portrait (1200x1600 on the 13.3"). |
| `POLL_MINUTES` | `15` | 0 to 255 | Minutes the panel sleeps between checks, sent as `Cache-Control: max-age` on `/panel`. `0` sends `no-cache`: as soon as the panel likes. |
| `QUIET_HOURS` | unset | `start-end`, hours | Local hours, e.g. `22-6`, that a panel sleeps through in one go: a panel asking during them is told to sleep until they end. Uses `UTC_OFFSET_MINUTES`. |
| `WINDOW_HOURS` | `24` | hours | How far back detections count. Rounded outwards to whole days, as BirdNET-Go's summary takes dates. |
| `UTC_OFFSET_MINUTES` | `0` | minutes | The garden's time zone, minutes east of UTC. Used for the date range and the "Updated" time. Not adjusted for daylight saving. |
| `REFRESH_SECONDS` | `60` | seconds | How often the server asks BirdNET-Go for news. |
| `ART_DIR` | unset | folder | A fugleramme art style folder (`birds/*.webp`, `geometry.json`), e.g. from `scripts/fetch-art.sh`. Unset: no pictures, so collages are drawn as the list. Species without a plate are left off the collage. |
| `LAYOUT` | `collage` | `collage` or `list` | `collage` packs a picture of every bird by its outline, heavier birds bigger. `list` writes names and counts only. |
| `MAX_BIRDS` | `0` | number | Most birds on a page, most heard first. `0` shows every bird heard, as fugleramme does. |
| `NAMES` | `true` | `true` or `false` | Whether the collage writes each bird's name under it. |
| `FACTS` | `true` | `true` or `false` | Whether today's, the week's and the rarest page end with one line about their main bird (the first one: a bird new for the year, the rarest, or the most heard), something special about it, a different one each day. From tables built into the server; nothing is looked up online. See [Pages and views](../explanation/pages-and-views.md#a-fact-about-the-main-bird). |
| `BORDER` | `season` | `season`, `off`, or a theme | The border of plants from period botanical plates in two corners of every page. `season` follows the date, with Swedish wild plants: spruce and juniper in winter, bird cherry and wood anemone in spring, cornflower and buttercup in summer, oak and bilberry in autumn, and mistletoe and lingonberry, pussy willow and coltsfoot, or catchfly and harebell on those holidays. A theme's name (`frost`, `blossom`, `meadow`, `autumn`, `christmas`, `easter`, `midsummer`) keeps that one all year. A negative `LATITUDE` turns the seasons round. See [Pages and views](../explanation/pages-and-views.md#seasonal-borders). |
| `HEADING` | unset | text | A heading over the page. Unset: none on the collage (birds only, like fugleramme), the translated "Heard in the garden" on the list. |
| `LANGUAGE` | `en` | language code | Language of the page's own words (heading, "No birds heard yet", "Updated 07:30"), e.g. `sv`, `nb`, `de`, `pt-BR`. A regional code falls back to its main language, then to English. Bird names come from BirdNET-Go in its own language setting. |
| `TRANSLATIONS` | unset | file | A translations JSON adding languages or overriding words; see below. |
| `VIEW` | `today` | `today`, `week`, `rarest`, `story`, `season` | The page frames show when they do not ask for one with `view=`. See [Pages and views](../explanation/pages-and-views.md). |
| `FRAMES` | unset | frames | Frames with settings of their own, chosen by `frame=`; see [Frames](#frames). |
| `LATITUDE`, `LONGITUDE` | unset | degrees | Where the garden is, for the weather and sunrise on the story page, from [Open-Meteo](https://open-meteo.com/) (free, no key). Set both or neither; unset leaves the weather off. With `BIRDNET_URL=mock` the simulated garden has weather of its own. |
| `WEATHER_URL` | `http://api.open-meteo.com` | URL | Where Open-Meteo is, e.g. a self-hosted copy. Plain HTTP: the server speaks no TLS. |
| `ALERT_POLL_MINUTES` | `5` | 0 to 255 | Minutes frames sleep while a bird new for the year is about, instead of `POLL_MINUTES`. `0` turns the shorter sleep off; the bird is still featured. |
| `ALERT_MINUTES` | `60` | minutes | How long after a new bird was last heard the shorter sleep lasts. |
| `ALMANAC_MINUTES` | `30` | minutes | How often the week, the year, last year's week, the hourly counts and the weather are asked for again. They are also asked for whenever today's birds change. |
| `MQTT_URL` | unset | `mqtt://user:password@host:port` | An MQTT broker to announce frames to Home Assistant on; the login and port (1883) may be left out. See [Show frames in Home Assistant](../how-to/show-frames-in-home-assistant.md). The password is never printed. |
| `MQTT_DISCOVERY_PREFIX` | `homeassistant` | text | Home Assistant's MQTT discovery prefix. |
| `FIRMWARE_DIR` | unset | folder | A folder holding `ee02-thin.bin` and `ee02-thin.version` to offer EE02 frames over Wi-Fi. The version must be the one built into the image. See [Set up and update the EE02 over Wi-Fi](../how-to/set-up-ee02-over-wifi.md). |
| `MOCK_DATE` | unset | `YYYY-MM-DD` | With `BIRDNET_URL=mock`, the day the simulated garden pretends it is, keeping the time of day: to see spring's arrivals or a rare bird in November. |

## Frames

`FRAMES` names frames and what each shows, so a big collage in the hall and a small list on the
desk can share one server. Frames are separated by `;`; each is a name (`a-z`, `0-9` and `-`, at
most 32), `:`, and settings separated by `,`:

```text
FRAMES=hall:layout=collage;desk:layout=list,panel=7.3,rotation=90,view=story,max_birds=12
```

| Setting | Values | Default |
| --- | --- | --- |
| `layout` | `collage`, `list` | `LAYOUT` |
| `view` | `today`, `week`, `rarest`, `story`, `season` | `VIEW` |
| `panel` | `13.3`, `7.3` | `PANEL` |
| `rotation` | `0`, `90`, `180`, `270` | `ROTATION` |
| `max_birds` | number, `0` for all | `MAX_BIRDS` |
| `names` | `true`, `false` | `NAMES` |
| `heading` | text without `,` or `;` | `HEADING` |
| `facts` | `true`, `false` | `FACTS` |
| `border` | `season`, `off`, or a theme | `BORDER` |

A frame asks with `frame=<name>`, set on its setup page. A frame that gives no name, or a name not
in `FRAMES`, gets the server's own settings.

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
| `heading_week`, `heading_rarest` | Headings of the week and rarest pages | |
| `new_for_year` | Heading when a bird is new for the year | `{name}` |
| `story_heading` | Heading of the story page | |
| `first_song`, `last_song` | The day's first and latest singer | `{time}`, `{name}` |
| `chorus_peak` | The busiest hour | `{time}` |
| `whole_chorus` | The chart's row of every bird together | |
| `weather` | The weather line | `{sky}`, `{now}`, `{low}`, `{high}` |
| `sunrise`, `sunset` | Marks on the chart | |
| `sky_clear`, `sky_partly_cloudy`, `sky_overcast`, `sky_fog`, `sky_drizzle`, `sky_rain`, `sky_snow`, `sky_thunder`, `sky_unknown` | What the sky is doing | |
| `fact_line` | The line about the main bird; `{fact}` is a note, or one of the numbers below | `{name}`, `{fact}` |
| `fact_weight`, `fact_weight_one` | Its weight, in sugar cubes (`_one` for about one) | `{weight}`, `{count}` |
| `fact_wingspan` | Its wingspan | `{cm}` |
| `fact_incubation` | Days its eggs take to hatch | `{days}` |
| `fact_oldest` | The oldest one known | `{years}` |
| `season_heading` | Heading of the season page | |
| `year_count` | Species this year | `{count}` |
| `arrivals`, `no_arrivals` | The arrivals column, and its line when it is empty | |
| `last_year`, `now_then` | The year-on-year column, and its first line | `{now}`, `{then}` |
| `gained`, `missing` | Beside birds heard only now, or only last year | |
| `year_list` | Title of the year list | |
| `months` | Month abbreviations, January first, separated by spaces | |

The words of the story and season pages are in English and Swedish; other languages show them in
English until a translations file adds them.

An invalid value stops the server with a message naming the variable, e.g.
`ROTATION: cannot read "45": rotation must be 0, 90, 180 or 270`.

## `fake-birdnet`

| Setting | Default | Meaning |
| --- | --- | --- |
| first argument | `127.0.0.1:8090` | Address to listen on |
| `UTC_OFFSET_MINUTES` | `0` | Time zone of the simulated garden |
