#!/bin/bash
set -eu

cat .lints | cut -f1 -d"#" | tr '\n' ' '
