#!/bin/bash
set -xeu

# Used to avoid failures of compilation tests because of multiple compiled versions of the same library
rm -rf target/debug/deps/*modor*
