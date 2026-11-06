# Border sprays

The plants in the page's border are Swedish wild plants, cut from the hand-coloured plates of
C. A. M. Lindman, *Bilder ur Nordens Flora* (third edition, Wahlström & Widstrand, Stockholm,
1917 to 1926; first published 1901 to 1905). The plates are in the **public domain**: Lindman
died in 1928, and the book was published more than 95 years ago. The scans are Project
Runeberg's ([runeberg.org/nordflor](https://runeberg.org/nordflor/)), which publishes them
freely; no licence is needed, and this credit is a courtesy.

| Spray | Plant | Plate |
| --- | --- | --- |
| `frost-top` | Spruce, *Picea abies* (gran) | [495](https://runeberg.org/nordflor/495.html), `pics/495.jpg` |
| `frost-bottom` | Juniper, *Juniperus communis* (en) | [497](https://runeberg.org/nordflor/497.html), `pics/497.jpg` |
| `blossom-top` | Bird cherry, *Prunus padus* (hägg) | [312](https://runeberg.org/nordflor/312.html), `pics/312.jpg` |
| `blossom-bottom` | Wood anemone, *Anemone nemorosa* (vitsippa) | [170](https://runeberg.org/nordflor/170.html), `pics/170.jpg` |
| `meadow-top` | Cornflower, *Centaurea cyanus* (blåklint) | [7](https://runeberg.org/nordflor/7.html), `pics/7.jpg` |
| `meadow-bottom` | Meadow buttercup, *Ranunculus acris* (smörblomma) | [159](https://runeberg.org/nordflor/159.html), `pics/159.jpg` |
| `autumn-top` | Pedunculate oak, *Quercus robur* (ek) | [368](https://runeberg.org/nordflor/368.html), `pics/368.jpg` |
| `autumn-bottom` | Bilberry, *Vaccinium myrtillus* (blåbär) | [141](https://runeberg.org/nordflor/141.html), `pics/141.jpg` |
| `christmas-top` | Mistletoe, *Viscum album* (mistel) | [363](https://runeberg.org/nordflor/363.html), `pics/363.jpg` |
| `christmas-bottom` | Lingonberry, *Vaccinium vitis-idaea* (lingon) | [143](https://runeberg.org/nordflor/143.html), `pics/143.jpg` |
| `easter-top` | Goat willow, *Salix caprea* (sälg) | [377](https://runeberg.org/nordflor/377.html), `pics/377.jpg` |
| `easter-bottom` | Coltsfoot, *Tussilago farfara* (hästhov) | [23](https://runeberg.org/nordflor/23.html), `pics/23.jpg` |
| `midsummer-top` | Sticky catchfly, *Viscaria vulgaris* (tjärblomster) | [341](https://runeberg.org/nordflor/341.html), `pics/341.jpg` |
| `midsummer-bottom` | Harebell, *Campanula rotundifolia* (liten blåklocka) | [53](https://runeberg.org/nordflor/53.html), `pics/53.jpg` |

`cut_sprays.py` made the `.webp` files from the scans: it keeps each plate's main plant (leaving
out the numbered details and captions), makes the paper transparent, tones the plate's paper to
the page's, and turns the plant to lie in its corner. Nothing else is changed or added.
