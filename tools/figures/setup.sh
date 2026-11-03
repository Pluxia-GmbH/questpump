#!/bin/sh
# Setup of the figure build on the Mac that renders (docs/plan.md slice 10): the Python venv of the pack
# build, inside tools/figures (.venv/, gitignored); re-running keeps the venv and brings its packages to
# requirements.txt's pins, so run it again after a pin there moves (pip leaves a package that already matches alone).
# Also needed, from Homebrew: blender (5.2 LTS), imagemagick, jpeg-xl (ssimulacra2), oxipng, pngquant, ffmpeg.
set -eu
cd "$(dirname "$0")"

for tool in blender magick ssimulacra2 oxipng pngquant ffmpeg; do
	command -v "$tool" >/dev/null || { echo "missing: $tool (brew install it)" >&2; exit 1; }
done

[ -x .venv/bin/python ] || python3 -m venv .venv
.venv/bin/pip install --quiet --disable-pip-version-check -r requirements.txt
echo "figure tools ready"
