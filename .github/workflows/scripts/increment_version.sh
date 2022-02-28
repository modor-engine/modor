#!/bin/bash
set -xeu

version="$1"
new_version=$(semver -i patch "$version")
IFS=";"
for crate_name in $PUBLISHED_CRATES; do
    cd "./crates/$crate_name"
    sed -i "s/^version \?=.*$/version = \"$new_version\"/g" Cargo.toml
    cd - || exit 1
done
