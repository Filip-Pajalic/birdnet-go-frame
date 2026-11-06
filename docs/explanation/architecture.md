# Architecture: how the pieces fit

All the heavy work happens on a machine that is always on (your Unraid box). The panel is a thin,
battery-powered display that downloads a finished picture only when it has changed.

```mermaid
flowchart LR
    mic([Microphone]) --> bng

    subgraph unraid[Unraid or any Docker host]
        bng[BirdNET-Go<br/>listens and classifies]
        srv[birdnet-go-frame server<br/>apps/server]
        bng -- "JSON over /api/v2<br/>every REFRESH_SECONDS" --> srv
    end

    subgraph frame[Picture frame]
        esp[ESP32-S3<br/>Seeed XIAO EE02]
        panel[13.3in Spectra 6<br/>e-ink panel]
        esp -- SPI --> panel
    end

    srv -- "/frame.e6<br/>or 304 Not Modified" --> esp
    srv -- "/preview.png" --> browser([Browser])
```

## What each part does

| Part | Runs on | Job |
| --- | --- | --- |
| BirdNET-Go | Unraid | Detects birds from audio. birdnet-go-frame only reads its HTTP API. |
| Server (`apps/server`) | Unraid | Polls BirdNET-Go, redraws the page when the set of species changes, serves it. |
| Panel firmware | ESP32-S3 | Wakes on a timer, asks for the page with its ETag, redraws only on a change, sleeps. |

## One cycle, step by step

```mermaid
sequenceDiagram
    autonumber
    participant B as BirdNET-Go
    participant S as Server
    participant P as Panel (ESP32)

    loop every REFRESH_SECONDS
        S->>B: GET /api/v2/analytics/species/summary
        B-->>S: species and counts (JSON)
        alt species set changed
            S->>S: lay out, draw, dither, pack (new ETag)
        else unchanged
            S->>S: keep the current page
        end
    end

    Note over P: deep sleep for POLL_MINUTES
    P->>S: GET /frame.e6 (If-None-Match: last ETag)
    alt page unchanged
        S-->>P: 304 Not Modified
        Note over P: back to sleep within seconds
    else new page
        S-->>P: 200, 20-byte header + 960,000 bytes
        P->>P: check CRC-32, Wi-Fi off, refresh panel (~20-30 s)
        Note over P: back to sleep
    end
```

## Why the work is split this way

- **Rendering needs resources a microcontroller lacks**: fonts, art, and room for a full-colour
  page. The server has all of them for free.
- **The panel's cost is time with the radio on.** Handing it a finished, checksummed picture keeps
  each wake short, and most wakes end in a 304 with no body at all.
- **The format is shared code**, not a spec written twice: `frame-core` is compiled into both the
  server and the firmware (see [Design decisions](design-decisions.md)).

Because every piece is a library, other arrangements are possible too, such as a Raspberry Pi that
renders and drives the panel itself, or an ESP32 that talks to BirdNET-Go directly. See
[Crates and apps](../reference/crates.md).
