#!/bin/bash
set -xeu

total_lines=$(grep -ci "^DA" lcov.info | tr -d '\n' || echo 0)
untested_lines=$(grep -ci "^DA.*,0$" lcov.info | tr -d '\n' || echo 0)
failure=$(awk "BEGIN{ print ($total_lines - $untested_lines) * 100 / $total_lines < $COV_THRESHOLD }")
if [ "$failure" -eq "1" ]; then
    echo "Coverage has failed with less than $COV_THRESHOLD%."
    exit 1
else
    echo "Coverage has successfully passed."
fi
