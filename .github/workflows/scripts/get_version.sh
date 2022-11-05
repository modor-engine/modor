#!/bin/bash
set -xeu

version=""
while read -r crate_name; do
    cd "./crates/$crate_name"
    current_version=$(grep -m 1 '^version' Cargo.toml | cut -d '"' -f2 | tr -d '\n')
    if [ "$version" == "" ]; then
       version="$current_version"
    elif [ "$current_version" != "$version" ]; then
        echo "All crates must have the same version"
        exit 1
    fi
    cd - || exit 1
done <PUBLISHED-CRATES

if [ "$version" == "" ]; then
    echo "Version not found in Cargo.toml files"
    exit 1
else
    echo "version=$version" >> "$GITHUB_OUTPUT"
fi
