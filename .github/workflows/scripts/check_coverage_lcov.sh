#!/bin/bash
set -xeu

total_lines=$(grep -ci "^DA" lcov.info)
untested_lines=$(grep -ci "^DA.*,0$" lcov.info)
failure=$(awk "BEGIN{ print ($total_lines - $untested_lines) * 100 / $total_lines < $COV_THRESHOLD }")
if [ "$failure" -eq "1" ]; then
  echo "Coverage has failed with $coverage% instead of at least $COV_THRESHOLD%."
  exit 1
else
  echo "Coverage has successfully passed with $coverage%."
fi
