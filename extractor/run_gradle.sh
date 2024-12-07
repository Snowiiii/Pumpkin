#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(dirname "$0")"

mkdir -p "$SCRIPT_DIR/run"
echo "eula=true" >"$SCRIPT_DIR/run/eula.txt"
(cd "$SCRIPT_DIR" && sh "./gradlew" "runServer")
