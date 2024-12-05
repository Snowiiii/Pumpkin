#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"
EXTRACTOR_DIR="./extractor"
MAPPER_FILE="$EXTRACTOR_DIR/file_map.dat"

while read -r line; do
    if [ -z "$line" ]; then
        continue
    fi

    if [[ "$line" == \#* ]]; then
        continue
    fi

    line_array=($line)
    first="${line_array[0]}"
    second="${line_array[1]}"

    dest_dir="$(dirname "$second")"
    mkdir -p "$SCRIPT_DIR/$dest_dir"
    mv "$EXTRACTOR_DIR/$first" "$SCRIPT_DIR/$second"
done <"$MAPPER_FILE"
