#!/bin/bash

set -euo pipefail

LINUX="linux"
MAC="mac"

UNAME="$(uname -s)"
case "${UNAME}" in
Linux*) machine="$LINUX" ;;
Darwin*) machine="$MAC" ;;
*) machine="UNKNOWN:${unameOut}" ;;
esac

echo "Moving assets for a $machine machine"

SCRIPT_DIR="$(realpath "$(dirname "$0")")"
ROOT_DIR="$SCRIPT_DIR/.."
EXTRACTOR_DIR="$ROOT_DIR/extractor"
MAPPER_FILE="$EXTRACTOR_DIR/file_map.dat"
SHA_FILE="$EXTRACTOR_DIR/expected_sha256.dump"
OUTPUT_DIR="$EXTRACTOR_DIR/run/pumpkin_extractor_output"

if [ "$machine" = "$MAC" ]; then
    (cd "$OUTPUT_DIR" && shasum -a 256 -c "$SHA_FILE")
else
    (cd "$OUTPUT_DIR" && sha256sum -c "$SHA_FILE")
fi

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

    dest_dir="$(dirname "$ROOT_DIR/$second")"
    mkdir -p "$dest_dir"
    mv "$OUTPUT_DIR/$first" "$ROOT_DIR/$second"
done <"$MAPPER_FILE"

echo "Sucessfully moved asset files!"
