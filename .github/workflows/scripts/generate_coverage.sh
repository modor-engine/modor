#!/bin/bash
set -xeu

output_type=$1
output_path=$2

ignore_args=$(echo $UNTESTED_CRATES | sed -r "s|([^;]*)|--ignore ./crates/\1|g" | sed "s|;| |g")

grcov . $ignore_args \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-type "$output_type" \
    --branch \
    --ignore-not-existing \
    --output-path "$output_path" \
    --excl-line '(#\[|^[^ ]+!\()' \
    --excl-start '#\[cfg\(test\)\]' \
    --keep-only "**/src/**/*"
