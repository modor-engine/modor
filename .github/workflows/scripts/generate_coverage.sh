#!/bin/bash
set -xeu

output_type=$1
output_path=$2

grcov . \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-type "$output_type" \
    --branch \
    --ignore-not-existing \
    --output-path "$output_path" \
    --excl-line '(#\[|^[^ ]+!\(|struct |impl( |<)|unreachable!|^\/\/\/|no-coverage|[a-z0-9_]: .+,)' \
    --excl-start 'coverage: off' \
    --excl-stop 'coverage: on' \
    --keep-only "**/crates/**/src/**/*"
