#!/bin/bash
set -xeu

grcov . \
    --binary-path ./target/debug/ \
    --source-dir . \
    --output-type lcov \
    --branch \
    --ignore-not-existing \
    --output-path ./lcov.info \
    --excl-line "#\[" \
    --excl-start "#\[cfg\(test\)\]" \
    --keep-only "**/src/**/*"
