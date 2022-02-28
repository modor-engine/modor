#!/bin/bash
set -xeu

IFS=";"
for crate_name in $PUBLISHED_CRATES; do
    cd "./crates/$crate_name"
    version=$(grep -m 1 '^version' Cargo.toml | cut -d '"' -f2 | tr -d '\n')
    if cargo search "$crate_name" --limit 1 | grep "^$crate_name = \"$version\""; then
        echo "'$crate_name' already published"
    else
        cargo publish --token "$CRATES_IO_TOKEN"
    fi
    cd - || exit 1
    sleep 60 # avoid reaching Cargo rate limit
done
