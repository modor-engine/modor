#!/bin/bash
set -xeu

for crate_path in crates/*; do
    crate_name=$(basename "$crate_path")
    grep -iozP "$crate_name\n" PUBLISHED-CRATES \
        || (echo "Crate '$crate_name' will not be published" && exit 1)
done
