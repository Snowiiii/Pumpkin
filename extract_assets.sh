#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
EXTRACTOR_DIR="./extractor"

cd "$SCRIPT_DIR"
if [ ! -d "$EXTRACTOR_DIR" ]; then
    echo "make sure to run this script as-is in the Pumpkin source root!"
    exit 1
fi

bash "$EXTRACTOR_DIR/run_gradle.sh"
bash "$SCRIPT_DIR/move_assets.sh"

rm -rf "$EXTRACTOR_DIR/run"
