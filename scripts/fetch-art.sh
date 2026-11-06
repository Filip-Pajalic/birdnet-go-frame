#!/bin/sh
# Downloads one of fugleramme's art styles (default: classic) into art/<style>, for ART_DIR.
#
# The artwork is not part of this repository. It is fugleramme's curation of 1800s bird plates,
# offered under CC BY-SA 4.0; the style folder keeps its ATTRIBUTION.md and manifest.json, which
# name the source of every plate. Credit the artwork if you share pictures of your frame.
#
# Usage: scripts/fetch-art.sh [style] [destination]
#   scripts/fetch-art.sh                 # art/classic
#   scripts/fetch-art.sh classic /mnt/user/appdata/birdnet-go-frame/art
set -eu

style="${1:-classic}"
destination="${2:-art}"
repository="https://github.com/arnegiacomo/fugleramme.git"

if ! command -v git >/dev/null 2>&1; then
    echo "fetch-art: git is required" >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

echo "fetch-art: downloading the '$style' style from fugleramme (about 250 MB)"
git clone --quiet --depth 1 --filter=blob:none --sparse "$repository" "$work/fugleramme"
git -C "$work/fugleramme" sparse-checkout set "assets/artwork/$style"

source_folder="$work/fugleramme/assets/artwork/$style"
if [ ! -d "$source_folder/birds" ]; then
    echo "fetch-art: fugleramme has no style called '$style'" >&2
    exit 1
fi

mkdir -p "$destination"
rm -rf "${destination:?}/$style"
cp -R "$source_folder" "$destination/$style"
count="$(find "$destination/$style/birds" -name '*.webp' | wc -l | tr -d ' ')"
echo "fetch-art: $count plates in $destination/$style"
echo "fetch-art: run the server with ART_DIR=$destination/$style"
