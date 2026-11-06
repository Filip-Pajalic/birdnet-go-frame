# Bird facts

The line about a page's main bird tells something special about it: a habit, a song, a record
or a piece of folklore from `notes_en.csv` and `notes_sv.csv`. For a bird without notes, or a
page in another language, it tells a number from `bird_facts.csv` instead. The frame never
looks anything up online: the tables are compiled in. Body mass comes from `bird-sizes`
(AVONET).

## Notes

`notes_en.csv` and `notes_sv.csv` were written for this project, in English and Swedish, for
about 140 birds common in Sweden. Each row is a scientific name and its notes, separated by
`|`, at most 72 characters each; rows are sorted by name ignoring case and spacing. They are
under the project's own licence.

## Numbers

| Column | Meaning | Source |
| --- | --- | --- |
| `wingspan_cm` | Wingspan, tip to tip | Wikidata P2050 |
| `incubation_days` | Days the eggs take to hatch | Wikidata P7770 |
| `oldest_years` | Longest observed lifespan | Wikidata P4214 |

## Sources

| | AVONET | Wikidata |
| --- | --- | --- |
| Source | Tobias, J.A. et al. (2022). *AVONET: morphological, ecological and geographical data for all birds.* Ecology Letters 25, 581–597. [doi:10.1111/ele.13898](https://doi.org/10.1111/ele.13898), file `AVONET2_eBird.xlsx` from [figshare](https://figshare.com/articles/dataset/AVONET_morphological_ecological_and_geographical_data_for_all_birds_Tobias_et_al_2021_Ecology_Letters_/16586228) | [Wikidata](https://www.wikidata.org/), queried on 2026-11-06 by each species' taxon name (P225) |
| Licence | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |

AVONET's terms ask that it is cited as above. No other conditions apply to pages drawn with it.

## How it was made

`build_bird_facts.py` (Python 3 with `openpyxl`) reads AVONET's eBird sheet, a CSV of the
Wikidata quantities, and `crates/bird-sizes/data/bird_sizes.csv`, and writes one row for each
species in the latter (BirdNET v2.4's labels, eBird names) that has any fact. AVONET's wing
lengths are only used to check the wingspans.

Changes: Wikidata values are converted to centimetres, days and years, and the median taken
where a species has several. Values outside plausible bounds (wingspans of 5 to 400 cm that are
1.6 to 4.5 times AVONET's wing length, incubation of 8 to 90 days, lifespans of 1 to 100 years)
are dropped as typos. Numbers are rounded to whole units.
