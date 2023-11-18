#!/bin/bash
set -xeuo pipefail

while IFS= read -r -d '' file; do
    matches=$((cat "$file" | awk 'tolower($0) ~ /todo/ && !/no-todocheck/') || exit 1)
    if [ ! -z "$matches" ]; then
        echo "TODO found in $file"
        exit 1
    fi
done < <(find . -type f \( -name '*.rs' -o -name '*.toml' \) -print0)
echo "No TODO found in code"
