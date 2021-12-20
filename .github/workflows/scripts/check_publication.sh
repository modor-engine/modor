#!/bin/bash
set -xeu

IFS=";"
for crate_path in $CRATE_PATHS; do
    cd "$crate_path"
    cargo publish --dry-run
    cd - || exit 1
done
