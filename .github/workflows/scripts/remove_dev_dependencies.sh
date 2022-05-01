#!/bin/bash
set -xeu

for crate_path in ./crates/*; do
    sed -e "/\[dev-dependencies\]/,/^$/d" "$crate_path/Cargo.toml" > "$crate_path/Cargo2.toml"
    cat "$crate_path/Cargo2.toml" > "$crate_path/Cargo.toml"
    rm "$crate_path/Cargo2.toml"
done
