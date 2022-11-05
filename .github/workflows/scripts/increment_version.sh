#!/bin/bash
set -xeu

version="$1"
new_version=$(semver -i patch "$version")
sed -i "s/^version \?=.*$/version = \"$new_version\"/g" Cargo.toml

