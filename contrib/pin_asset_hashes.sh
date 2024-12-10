#!/bin/bash

# This isn't necessary to build pumpkin, so I'm not going to make one for windows

set -euo pipefail

SCRIPT_DIR="$(realpath $(dirname "$0"))"
ROOT_DIR="$SCRIPT_DIR/.."
EXTRACTOR_DIR="$ROOT_DIR/extractor"

if [ ! -d "$EXTRACTOR_DIR" ]; then
    echo "make sure to run this script as-is in the Pumpkin source root!"
    exit 1
fi

bash "$EXTRACTOR_DIR/run_gradle.sh"
ASSET_DIR="$EXTRACTOR_DIR/run/pumpkin_extractor_output"

(cd "$ASSET_DIR" && sha256sum * >"$EXTRACTOR_DIR/expected_sha256.dump")

rm -rf "$EXTRACTOR_DIR/run"
