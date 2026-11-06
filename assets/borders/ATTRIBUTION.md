# Border sprays

The plants in the page's border are cut from the hand-coloured plates of Otto Wilhelm Thomé,
*Flora von Deutschland, Österreich und der Schweiz in Wort und Bild für Schule und Haus*
(Gera, 1885), drawn by Walter Müller. The plates are in the **public domain**: Thomé died in
1885 and Müller in 1927, and the book was published more than 95 years ago. The scans are from
Wikimedia Commons, which marks them public domain; no licence is needed, and this credit is a
courtesy.

| Spray | Plant | Commons file |
| --- | --- | --- |
| `frost-top`, `christmas-bottom` | Mistletoe, *Viscum album* | [Illustration_Viscum_album0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Viscum_album0.jpg) |
| `frost-bottom` | Christmas rose, *Helleborus niger* | [Illustration_Helleborus_niger0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Helleborus_niger0.jpg) |
| `blossom-top` | Bird cherry, *Prunus padus* (Thomé's *Padus avium*) | [Illustration_Prunus_avium0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Prunus_avium0.jpg) |
| `blossom-bottom`, `easter-bottom` | Wood anemone, *Anemone nemorosa* | [Illustration_Anemone_nemorosa0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Anemone_nemorosa0.jpg) |
| `meadow-top`, `midsummer-bottom` | Cornflower, *Centaurea cyanus* | [Illustration_Centaurea_cyanus0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Centaurea_cyanus0.jpg) |
| `meadow-bottom`, `midsummer-top` | Ox-eye daisy, *Leucanthemum vulgare* | [Illustration_Chrysanthemum_leucanthemum0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Chrysanthemum_leucanthemum0.jpg) |
| `autumn-top` | Rowan, *Sorbus aucuparia* | [Illustration_Sorbus_aucuparia0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Sorbus_aucuparia0.jpg) |
| `autumn-bottom` | Pedunculate oak, *Quercus robur* | [Illustration_Quercus_robur0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Quercus_robur0.jpg) |
| `christmas-top` | Holly, *Ilex aquifolium* | [Illustration_Ilex_aquifolium0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Ilex_aquifolium0.jpg) |
| `easter-top` | Goat willow, *Salix caprea* | [Illustration_Salix_caprea0.jpg](https://commons.wikimedia.org/wiki/File:Illustration_Salix_caprea0.jpg) |

`cut_sprays.py` made the `.webp` files from the scans: it keeps each plate's main plant (leaving
out the numbered details and captions), makes the paper transparent, tones the plate's paper to
the page's, and turns the plant to lie in its corner. Nothing else is changed or added.
