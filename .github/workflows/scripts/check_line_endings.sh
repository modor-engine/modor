#!/bin/bash
set -xeu

shopt -s globstar

check_file_has_lf_endings() {
    if [ -f "$1" ]; then
        dos2unix <"$1" | cmp - "$1"
    fi
}

for file in ./**/*.{rs,toml,sh,txt,yml,yaml,md}; do
    check_file_has_lf_endings "$file"
done
