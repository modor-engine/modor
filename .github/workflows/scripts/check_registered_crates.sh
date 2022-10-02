#!/bin/bash
set -xeu

cargo update
for crate_path in crates/*; do
    crate_name=$(basename "$crate_path")
    cargo search --limit 5
    cargo search modor --limit 1
    (cargo search a --limit 1 | grep "^a = ")\
        || (echo "Crate a not registered on crates.io" && exit 1)
    (cargo search modor --limit 1 | grep "^modor = ")\
        || (echo "Crate modor not registered on crates.io" && exit 1)
    (cargo search "$crate_name" --limit 1 | grep "^$crate_name = ")\
        || (echo "Crate '$crate_name' not registered on crates.io" && exit 1)
done
