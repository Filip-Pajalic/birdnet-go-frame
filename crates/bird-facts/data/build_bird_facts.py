#!/usr/bin/env python3
"""Builds bird_facts.csv from AVONET and Wikidata. See ATTRIBUTION.md.

Usage: build_bird_facts.py AVONET2_eBird.xlsx AVONET1_BirdLife.csv wikidata.csv bird_sizes.csv

wikidata.csv has the columns name,prop,amount,unit: one row per quantity statement of P2050
(wingspan), P7770 (egg incubation period) and P4214 (longest observed lifespan) on the item
whose P225 (taxon name) is the name. Needs openpyxl.
"""

import csv
import statistics
import sys

import openpyxl

NICHES = {
    "Invertivore": "I",
    "Granivore": "G",
    "Frugivore": "F",
    "Nectarivore": "N",
    "Herbivore terrestrial": "H",
    "Herbivore aquatic": "W",
    "Aquatic predator": "A",
    "Vertivore": "V",
    "Scavenger": "S",
    "Omnivore": "O",
}
LIFESTYLES = {
    "Aerial": "A",
    "Aquatic": "Q",
    "Terrestrial": "T",
    "Insessorial": "I",
    "Generalist": "G",
}
# Wikidata units to centimetres, days and years.
TO_CM = {"Q11573": 100.0, "Q174728": 1.0, "Q174789": 0.1}
TO_DAYS = {"Q573": 1.0, "Q11574": 1.0 / 86400.0}
TO_YEARS = {"Q577": 1.0, "Q11574": 1.0 / (365.25 * 86400.0)}


def number(text):
    try:
        return float(text)
    except (TypeError, ValueError):
        return None


def main(ebird_path, birdlife_path, wikidata_path, sizes_path):
    wanted = [row["scientific_name"] for row in csv.DictReader(open(sizes_path))]
    birdlife = {}
    for row in csv.DictReader(open(birdlife_path)):
        birdlife[row["Avibase.ID1"]] = row
        birdlife[row["Species1"]] = row
    sheet = openpyxl.load_workbook(ebird_path, read_only=True)["AVONET2_eBird"]
    rows = sheet.iter_rows(values_only=True)
    header = next(rows)
    avonet = {}
    for values in rows:
        row = dict(zip(header, values))
        avonet[row["Species2"]] = row
    quantities = {}
    for row in csv.DictReader(open(wikidata_path)):
        quantities.setdefault((row["name"], row["prop"]), []).append(
            (float(row["amount"]), row["unit"])
        )

    def wikidata(name, prop, units, low, high):
        values = [
            amount * units[unit]
            for amount, unit in quantities.get((name, prop), [])
            if unit in units
        ]
        values = [value for value in values if low <= value <= high]
        return statistics.median(values) if values else None

    out = csv.writer(sys.stdout, lineterminator="\n")
    out.writerow(
        [
            "scientific_name",
            "wingspan_cm",
            "incubation_days",
            "oldest_years",
            "range_kkm2",
            "north",
            "south",
            "migration",
            "diet",
            "lifestyle",
        ]
    )
    for name in wanted:
        row = avonet.get(name, {})
        wing_mm = number(row.get("Wing.Length"))
        span = wikidata(name, "P2050", TO_CM, 5.0, 400.0)
        # A wingspan that does not fit the wing's measured length is someone's typo.
        if span is not None and wing_mm and not 1.6 <= span * 10.0 / wing_mm <= 4.5:
            span = None
        incubation = wikidata(name, "P7770", TO_DAYS, 8.0, 90.0)
        oldest = wikidata(name, "P4214", TO_YEARS, 1.0, 100.0)
        # The same bird in BirdLife's taxonomy: by Avibase ID, else by name.
        range_row = birdlife.get(row.get("Avibase.ID2")) or birdlife.get(name, {})
        range_km2 = number(range_row.get("Range.Size"))
        north = number(range_row.get("Max.Latitude"))
        south = number(range_row.get("Min.Latitude"))
        migration = row.get("Migration")
        fields = [
            name,
            "" if span is None else str(round(span)),
            "" if incubation is None else str(round(incubation)),
            "" if oldest is None else str(round(oldest)),
            "" if range_km2 is None else str(max(1, round(range_km2 / 1000.0))),
            "" if north is None else str(round(north)),
            "" if south is None else str(round(south)),
            "" if migration not in (1, 2, 3) else str(migration),
            NICHES.get(row.get("Trophic.Niche"), ""),
            LIFESTYLES.get(row.get("Primary.Lifestyle"), ""),
        ]
        if any(fields[1:]):
            out.writerow(fields)


if __name__ == "__main__":
    main(*sys.argv[1:5])
