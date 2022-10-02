#!/bin/bash
set -xeu

while read -r crate_name; do
    cd "./crates/$crate_name"
    version=$(grep -m 1 '^version' Cargo.toml | cut -d '"' -f2 | tr -d '\n')
    if cargo search "$crate_name" --limit 1 | grep "^$crate_name = \"$version\""; then
        echo "'$crate_name' already published"
    else
        cargo publish --token "$CRATES_IO_TOKEN"
        sleep 60 # avoid reaching Cargo rate limit
    fi
    cd - || exit 1
done <PUBLISHED-CRATES
