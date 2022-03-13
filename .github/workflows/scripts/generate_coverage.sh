#!/bin/bash
set -xeu

output_type=$1
output_path=$2

for crate in $UNTESTED_CRATES; do
    rm -rf "./crates/$crate"
done

grcov . \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-type "$output_type" \
    --branch \
    --ignore-not-existing \
    --output-path "$output_path" \
    --excl-line '(#\[|^[^ ]+!\()' \
    --excl-start '#\[cfg\(test\)\]' \
    --keep-only "**/src/**/*"
