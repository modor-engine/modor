#!/bin/bash
set -xeu

RUSTFLAGS="-Zinstrument-coverage" cargo test --lib
grcov . \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --output-path ./coverage/ \
    --excl-line "#\[" \
    --excl-start "#\[cfg\(test\)\]" \
    --keep-only "src/**/*"
