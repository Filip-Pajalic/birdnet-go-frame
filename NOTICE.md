# Third-party notices

## fugleramme

Parts of this project are derived from fugleramme: https://github.com/arnegiacomo/fugleramme

- the controllers' pixel layout from the `/frame.e6` format, in `crates/frame-core/src/panel_bytes.rs`;
- the collage in `crates/frame-layout` (`collage.rs`, `footprint.rs`, `spiral.rs`), ported from
  `render/collage.py` and `render/packing.py`;
- the paper-halo blending in `crates/art-webp/src/halo.rs`, from `render/paper.py`, and the
  plate choice per day in `art_folder.rs`, from `names.py`;
- the panel boost in `crates/frame-raster/src/pop.rs`, from `render/dither.py`;
- the panel command sequence and refresh checks in `crates/panel-t133a01`, from `examples/ee02/main.py`;
- the EE02 wiring and the panel's power-up and power-down (load switch first, pins held low
  through deep sleep) in `crates/glass-esp/src/ee02.rs`, from the same file.

Files containing ported code say so in their header comment.

```
MIT License

Copyright (c) 2026 Arne Giacomo Munthe-Kaas

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### fugleramme artwork

Not distributed with this repository, except rendered example pages in `docs/images/`
(`collage-classic.jpg`, `view-today-new-bird.jpg`, `view-week.jpg`, `view-rarest.jpg`), which
show plates from the `classic` style and are shared under CC BY-SA 4.0 like them (sources: von Wright, *Svenska Fåglar*; John Gould, *The Birds of
Europe* and others, as listed in the style's `ATTRIBUTION.md`). When downloaded by `scripts/fetch-art.sh`, each style
folder keeps its own `ATTRIBUTION.md` and `manifest.json`, which link every cut-out to the
plate it came from. The `classic` style is licensed under CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/).

## Seeed_GFX2

The T133A01 panel initialisation sequence references Seeed_GFX2's `Driver_T133A01.cpp`,
https://github.com/Seeed-Studio/Seeed_GFX2, MIT License (see that repository's LICENSE file).

## Zephyr

The 7.3" panel's register values in `crates/panel-ed2208/src/commands.rs`, and its reset,
refresh and sleep order, follow Zephyr's ED2208 driver, `drivers/display/display_ed2208_gca.c`,
https://github.com/zephyrproject-rtos/zephyr, Copyright (c) 2026 The Zephyr Project
Contributors, Apache License 2.0 (https://www.apache.org/licenses/LICENSE-2.0). The pins of
Seeed's reTerminal E1002 panel, in `crates/glass-esp/src/e1002.rs`, come from that project's
board files.

## BirdNET-Go

Not included. This project only calls BirdNET-Go's HTTP API. BirdNET-Go is licensed under
CC BY-NC-SA 4.0: https://github.com/tphakala/birdnet-go

## Data

- AVONET body masses: Tobias, J.A. et al. (2022). AVONET: morphological, ecological and
  geographical data for all birds. Ecology Letters. doi:10.1111/ele.13898. CC BY 4.0.
  Included as `crates/bird-sizes/data/bird_sizes.csv`, taken from fugleramme's
  `assets/bird_sizes.csv`, filtered to the species in BirdNET's v2.4 labels and rounded to three
  significant figures (see the `ATTRIBUTION.md` beside it).
- AVONET range sizes, latitudes, migration, trophic niche and lifestyle (same citation and
  licence), from its `AVONET2_eBird.xlsx` and `AVONET1_BirdLife.csv`, and Wikidata's wingspans,
  incubation periods and longest observed lifespans (https://www.wikidata.org/, CC0 1.0):
  included as `crates/bird-facts/data/bird_facts.csv`, built by `build_bird_facts.py` beside it
  (see the `ATTRIBUTION.md` there for the changes made).
- OpenFauna scientific-name aliases: https://github.com/tphakala/openfauna, CC BY-SA 4.0.

## Border plants

The border's plants in `assets/borders/` are cut from the hand-coloured plates of Otto Wilhelm
Thomé's *Flora von Deutschland, Österreich und der Schweiz* (1885), as scanned on Wikimedia
Commons. Public domain; see `assets/borders/ATTRIBUTION.md` for each plate.

## Fonts

Fonts in `assets/fonts/` are licensed under the SIL Open Font License 1.1. Each font folder
contains its licence file.
