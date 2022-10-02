#!/bin/bash
set -xeu

cargo update
for crate_path in crates/*; do
    crate_name=$(basename "$crate_path")
    cargo search --limit 5
    (cargo search "$crate_name" --limit 1 | grep "^$crate_name = ")\
        || (echo "Crate '$crate_name' not registered on crates.io" && exit 1)
done
