# Bird facts

`bird_facts.csv` holds a few facts per species for the line about a page's main bird. The frame
never looks anything up online: the table is compiled in. Body mass comes from `bird-sizes`
(AVONET too).

| Column | Meaning | Source |
| --- | --- | --- |
| `wingspan_cm` | Wingspan, tip to tip | Wikidata P2050 |
| `incubation_days` | Days the eggs take to hatch | Wikidata P7770 |
| `oldest_years` | Longest observed lifespan | Wikidata P4214 |
| `range_kkm2` | Range size, thousands of km² | AVONET `Range.Size` (BirdLife taxonomy) |
| `north`, `south` | The range's extreme latitudes, south negative | AVONET `Max.Latitude`, `Min.Latitude` |
| `migration` | 1 resident, 2 partial migrant, 3 migrant | AVONET `Migration` |
| `diet` | Trophic niche: `I`nvertivore, `G`ranivore, `F`rugivore, `N`ectarivore, `H`erbivore terrestrial, herbivore aquatic (`W`), `A`quatic predator, `V`ertivore, `S`cavenger, `O`mnivore | AVONET `Trophic.Niche` |
| `lifestyle` | `A`erial, a`Q`uatic, `T`errestrial, `I`nsessorial, `G`eneralist | AVONET `Primary.Lifestyle` |

## Sources

| | AVONET | Wikidata |
| --- | --- | --- |
| Source | Tobias, J.A. et al. (2022). *AVONET: morphological, ecological and geographical data for all birds.* Ecology Letters 25, 581–597. [doi:10.1111/ele.13898](https://doi.org/10.1111/ele.13898), files `AVONET2_eBird.xlsx` and `AVONET1_BirdLife.csv` from [figshare](https://figshare.com/articles/dataset/AVONET_morphological_ecological_and_geographical_data_for_all_birds_Tobias_et_al_2021_Ecology_Letters_/16586228) | [Wikidata](https://www.wikidata.org/), queried on 2026-11-06 by each species' taxon name (P225) |
| Licence | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |

AVONET's terms ask that it is cited as above. No other conditions apply to pages drawn with it.

## How it was made

`build_bird_facts.py` (Python 3 with `openpyxl`) reads the AVONET files, a CSV of the Wikidata
quantities, and `crates/bird-sizes/data/bird_sizes.csv`, and writes one row for each species in
the latter (BirdNET v2.4's labels, eBird names) that has any fact. AVONET's eBird sheet is
joined to its BirdLife sheet by Avibase ID, else by name, for the range columns.

Changes: Wikidata values are converted to centimetres, days and years, and the median taken
where a species has several. Values outside plausible bounds (wingspans of 5 to 400 cm that are
1.6 to 4.5 times AVONET's wing length, incubation of 8 to 90 days, lifespans of 1 to 100 years)
are dropped as typos. Numbers are rounded to whole units.
