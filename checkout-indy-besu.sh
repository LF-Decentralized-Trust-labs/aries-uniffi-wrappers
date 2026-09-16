#!/bin/sh

set -eo pipefail

git submodule update --init --recursive --no-checkout
cd indy-besu
git sparse-checkout set vdr
git checkout
cd ..
