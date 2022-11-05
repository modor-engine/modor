#!/bin/bash
set -xeu

echo "RUST_VERSION_STABLE=$(grep -i rust-version Cargo.toml | cut -d '"' -f2 | tr -d '\n').0" >> "$GITHUB_OUTPUT"
