#!/bin/bash
set -xeu

version="$1"
new_version=$(semver -i patch "$version")
while read -r crate_name; do
    cd "./crates/$crate_name"
    sed -i "s/^version \?=.*$/version = \"$new_version\"/g" Cargo.toml
    cd - || exit 1
done <PUBLISHED-CRATES
