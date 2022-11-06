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
    --excl-line '(#\[|^[^ ]+!\(|struct |unreachable!)' \
    --excl-start '(#\[cfg\(test\)\]|coverage: off)' \
    --excl-stop 'coverage: on' \
    --excl-br-start '(trace!\(|debug!\(|info!\()' \
    --excl-br-stop '\);' \
    --keep-only "**/crates/**/src/**/*"
