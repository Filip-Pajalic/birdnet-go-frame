#!/usr/bin/env python3
"""Cuts the border sprays out of Thomé's botanical plates.

Each spray is the main plant of one plate from Otto Wilhelm Thomé, *Flora von Deutschland,
Österreich und der Schweiz* (1885), as scanned on Wikimedia Commons (public domain). The
plate's paper is made transparent, its loose detail drawings, numbers and captions are left
out, its paper tone is matched to the page's, and the plant is turned to lie in its corner.

Usage: cut_sprays.py PLATES_DIR OUT_DIR
PLATES_DIR holds the plates as `<Genus>_<species>.jpg`: the Commons files
`Illustration_<Genus>_<species>0.jpg`, at 1280 pixels wide (see ATTRIBUTION.md).
Needs Pillow, NumPy and SciPy. Not run by the build: the sprays it writes are committed.
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage

# The page's paper, as the renderer draws it (frame_raster::Renderer::PAPER).
PAGE_PAPER = np.array([233, 225, 202], dtype=np.float32)
# Longest side of a written spray, in pixels.
MAX_SIDE = 480

# (theme, corner, plate, a box (left, top, right, bottom, as fractions) round the plant's
# middle, boxes of other drawings to leave out where they touch it, degrees to turn
# anticlockwise, mirror). The top spray hangs from the top-left
# corner; the bottom one rises from the bottom-right.
SPRAYS = [
    ("frost", "top", "Viscum_album", (0.03, 0.22, 0.97, 0.93), [], 225, False),
    ("frost", "bottom", "Helleborus_niger", (0.22, 0.04, 0.72, 0.95), [], 45, False),
    ("blossom", "top", "Prunus_avium", (0.1, 0.1, 0.85, 0.9), [(0.88, 0.1, 1.0, 0.45)], 225, False),
    ("blossom", "bottom", "Anemone_nemorosa", (0.1, 0.17, 0.85, 0.92), [], 45, False),
    ("meadow", "top", "Centaurea_cyanus", (0.28, 0.03, 0.72, 0.92), [], 225, False),
    ("meadow", "bottom", "Chrysanthemum_leucanthemum", (0.1, 0.03, 0.95, 0.92), [], 45, False),
    ("autumn", "top", "Sorbus_aucuparia", (0.15, 0.1, 0.85, 0.9), [], 225, False),
    ("autumn", "bottom", "Quercus_robur", (0.3, 0.4, 0.9, 0.85), [(0.0, 0.0, 0.42, 0.5)], 45, False),
    ("christmas", "top", "Ilex_aquifolium", (0.04, 0.03, 0.96, 0.93), [], 225, False),
    ("christmas", "bottom", "Viscum_album", (0.03, 0.22, 0.97, 0.93), [], 45, True),
    ("easter", "top", "Salix_caprea", (0.05, 0.05, 0.97, 0.86), [], 225, False),
    ("easter", "bottom", "Anemone_nemorosa", (0.1, 0.17, 0.85, 0.92), [], 45, True),
    ("midsummer", "top", "Chrysanthemum_leucanthemum", (0.1, 0.03, 0.95, 0.92), [], 225, True),
    ("midsummer", "bottom", "Centaurea_cyanus", (0.28, 0.03, 0.72, 0.92), [], 45, True),
]


def paper_colour(pixels):
    """The plate's paper under each pixel: scans darken towards the binding and the edges, so
    it is found locally, as the lightest common colour round each pixel."""
    height, width = pixels.shape[:2]
    small = np.asarray(
        Image.fromarray(pixels.astype(np.uint8)).resize((width // 8, height // 8)),
        dtype=np.float32,
    )
    lightness = small.sum(axis=2)
    # Pixels as light as most of their neighbourhood's paper; ink is darker, so is left out.
    level = ndimage.percentile_filter(lightness, 90, size=25)
    paper = np.empty_like(small)
    near = lightness >= level - 30
    weight = ndimage.uniform_filter(near.astype(np.float32), 25) + 1e-3
    for channel in range(3):
        sums = ndimage.uniform_filter(small[:, :, channel] * near, 25)
        paper[:, :, channel] = sums / weight
    big = Image.fromarray(np.clip(paper, 0, 255).astype(np.uint8)).resize(
        (width, height), Image.Resampling.BILINEAR)
    return np.asarray(big, dtype=np.float32)


def plant_mask(pixels, paper, keep, drops):
    """Pixels that are the plant, not paper, with paper wholly enclosed by small loops of ink
    (petals, berries) filled in. The plant is the biggest piece of ink inside `keep` (left,
    top, right, bottom, as fractions), followed out past the box's edges so no leaf is cut
    off, but never into a `drops` box (a neighbouring drawing that touches it)."""
    height, width = pixels.shape[:2]
    distance = np.sqrt(((pixels - paper) ** 2).sum(axis=2))
    ink = distance > 42
    # A scan's shadows are grey and a little darker than the paper; the plates' greys are
    # darker still, their whites lighter.
    chroma = pixels.max(axis=2) - pixels.min(axis=2)
    darker = paper.mean(axis=2) - pixels.mean(axis=2)
    ink &= ~((chroma < 30) & (darker > 0) & (darker < 70))
    # So is the shadow along the scan's sides: the paper's own colour, only dimmer.
    ratio = pixels / np.maximum(paper, 1)
    spread = ratio.max(axis=2) - ratio.min(axis=2)
    dimmed = ratio.mean(axis=2)
    sides = np.zeros_like(ink)
    sides[:, :int(0.12 * width)] = sides[:, -int(0.12 * width):] = True
    ink &= ~(sides & (spread < 0.1) & (dimmed > 0.5) & (dimmed < 0.97))
    # The scan's own edge is never plant.
    edge = int(0.04 * width)
    ink[:edge] = ink[-edge:] = False
    ink[:, :edge] = ink[:, -edge:] = False
    for left, top, right, bottom in drops:
        ink[int(top * height):int(bottom * height), int(left * width):int(right * width)] = False
    ink = ndimage.binary_opening(ink, iterations=1)
    # Merge the plant's parts across small gaps.
    joined = ndimage.binary_dilation(ink, iterations=2)
    left, top, right, bottom = keep
    box = np.zeros_like(ink)
    box[int(top * height):int(bottom * height), int(left * width):int(right * width)] = True
    labels, count = ndimage.label(joined & box)
    if count == 0:
        raise ValueError("no ink")
    sizes = ndimage.sum(ink, labels, range(1, count + 1))
    seed = labels == (1 + int(np.argmax(sizes)))
    # The whole of every piece the seed belongs to, inside or outside the box.
    pieces, _ = ndimage.label(joined)
    whole = np.isin(pieces, np.unique(pieces[seed]))
    mask = ink & whole
    mask = ndimage.binary_closing(mask, iterations=2)
    holes, count = ndimage.label(ndimage.binary_fill_holes(mask) & ~mask)
    sizes = ndimage.sum(np.ones_like(mask), holes, range(1, count + 1))
    small = np.isin(holes, 1 + np.nonzero(sizes < width * height * 0.015)[0])
    return mask | small


def cut(path, keep, drops, degrees, mirror):
    image = Image.open(path).convert("RGB")
    pixels = np.asarray(image, dtype=np.float32)
    paper = paper_colour(pixels)
    mask = plant_mask(pixels, paper, keep, drops)
    # Tone the plate's paper to the page's, so the spray's edges sit on one sheet.
    toned = np.clip(pixels * (PAGE_PAPER / np.maximum(paper, 1)), 0, 255)
    alpha = ndimage.gaussian_filter(mask.astype(np.float32), 1.0)
    rgba = np.dstack([toned, alpha * 255]).astype(np.uint8)
    ys, xs = np.nonzero(mask)
    spray = Image.fromarray(rgba, "RGBA").crop((xs.min(), ys.min(), xs.max() + 1, ys.max() + 1))
    if mirror:
        spray = spray.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
    if degrees:
        spray = spray.rotate(degrees, resample=Image.Resampling.BICUBIC, expand=True)
        spray = spray.crop(spray.getbbox())
    spray.thumbnail((MAX_SIDE, MAX_SIDE), Image.Resampling.LANCZOS)
    return spray


def main():
    plates, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    for theme, corner, plate, keep, drops, degrees, mirror in SPRAYS:
        spray = cut(plates / f"{plate}.jpg", keep, drops, degrees, mirror)
        spray.save(out / f"{theme}-{corner}.webp", quality=80, method=6)
        print(theme, corner, plate, spray.size)


if __name__ == "__main__":
    main()
