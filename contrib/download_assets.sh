#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(realpath "$(dirname "$0")")"
ROOT_DIR="$SCRIPT_DIR/.."
EXTRACTOR_DIR="$ROOT_DIR/extractor"
OUTPUT_DIR="$EXTRACTOR_DIR/run/pumpkin_extractor_output"

if [ ! -d "$EXTRACTOR_DIR" ]; then
    echo "make sure to run this script as-is in the Pumpkin source root!"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"
curl -o "$OUTPUT_DIR/pumpkin-assets.zip" "https://pumpkin.kralverde.dev/assets/artifacts/pumpkin-assets.zip"
unzip "$OUTPUT_DIR/pumpkin-assets.zip" -d "$OUTPUT_DIR"

bash "$SCRIPT_DIR/move_assets.sh"

rm -rf "$EXTRACTOR_DIR/run"
