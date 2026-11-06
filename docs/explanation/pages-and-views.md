# Pages and views

A frame can show five pages. It asks for one with `view=`; the EE02's button cycles through them,
and each frame in `FRAMES` has a default. Every page is drawn for whatever panel or screen asks,
in the frame's layout where the page has birds on it.

| View | Shows | From BirdNET-Go |
| --- | --- | --- |
| `today` (default) | The birds heard in the last `WINDOW_HOURS`, as a collage or a list | The species summary |
| `week` | The birds heard in the last seven days | The summary over seven days |
| `rarest` | Today's birds, rarest first: at most six on the collage, the rarest biggest | Today's summary, ranked by detections this year |
| `story` | The day so far: first and latest singer, the chorus's busiest hour, the weather, and a chart of when each bird sang | Today's summary, the hourly counts, and Open-Meteo |
| `season` | The year so far: species this year, recent arrivals, this week against the same week last year, and the year list | Summaries over the year, the last weeks, and last year's week |

## Today, with a new bird

![The collage with a Bohemian Waxwing, new for the year, drawn biggest under its own heading](../images/view-today-new-bird.jpg)

When a bird is heard for the first time this year, it is the alert: today's and the week's pages
put it first and biggest, today's heading names it ("New this year: Bohemian Waxwing"), and the
story chart draws its row in red. While it was heard within `ALERT_MINUTES`, `/panel` tells frames
to sleep `ALERT_POLL_MINUTES` (5 minutes) instead of `POLL_MINUTES`, so a frame that wakes
on its usual schedule then keeps up while the bird is about. The server cannot wake a sleeping
frame; the shorter sleep takes effect from the frame's next wake.

"New for the year" counts from the 15th of January: on the first of January every bird is new,
and residents take a while to all be heard.

## Week and rarest

![The week's collage](../images/view-week.jpg)

![Today's rarest birds, the bullfinch biggest](../images/view-rarest.jpg)

Rarity is the station's own: the fewer times a bird has been heard this year, the rarer. A
bird BirdNET-Go hears every day is never rare, however rare it is nationally, and a bird that
turns up once in a while is, however common elsewhere.

## The day's story

![The story page on the 13.3-inch panel: four lines, then a bar chart per bird by hour, with sunrise and sunset marked](../images/view-story.png)

Each row is one of the day's eight most heard birds, plus any new for the year (in red, first);
the top row, in blue, is every bird together. Bars are scaled per row, so a quiet bird's
pattern shows as clearly as a loud one's: the chart is about *when*, not *how many*. The chart
spans the hours anything sang, sunrise and sunset, and at least ten hours. Rows that do not fit at
a legible size are left off from the bottom, so the 7.3-inch panel shows fewer:

![The story page on the 7.3-inch panel](../images/view-story-7.3.png)

![The story page in portrait, in Swedish](../images/view-story-portrait-sv.png)

The weather line and the sunrise and sunset marks come from [Open-Meteo](https://open-meteo.com/),
which needs no key, when `LATITUDE` and `LONGITUDE` are set. Without them the page leaves them
out.

## The season

![The season page: 39 species this year, three spring arrivals, this week against last year's, and the numbered year list](../images/view-season.png)

- **Just arrived** lists the birds heard in the last 21 days that were not heard in the 60 days
  before: spring's migrants, autumn's winter thrushes, and rarities.
- **This week last year** compares the last seven days with the same seven days a year
  earlier, and names a few birds heard only now or only then.
- **Year list** numbers every species heard this year in the order they were first heard, like
  a birder's notebook. A long list is set smaller, then cut short with "and N more".

## What it costs BirdNET-Go

Today's summary is asked for every `REFRESH_SECONDS`, as before. The rest (six more summaries,
the hourly counts, and the weather) only every `ALMANAC_MINUTES` (30) and when today's birds
change, so a new bird is featured within a minute of BirdNET-Go hearing it. The year's summary
is the heaviest query; BirdNET-Go answers it from its own database in well under a second on a
Raspberry Pi with a year of detections.

Pages are drawn when a frame first asks for them after the birds change, then kept, so a
frame that never asks for the season page never costs its drawing.
