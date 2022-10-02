#!/bin/bash
set -xeu

for crate_path in crates/*; do
    crate_name=$(basename "$crate_path")
    cargo search "$crate_name" --limit 1 | { if ! grep "^$crate_name = "; then
      echo "Crate '$crate_name' not registered on crates.io"
      exit 1
    fi }
done
