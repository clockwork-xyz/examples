#!/usr/bin/env bash

set -e

here="$(dirname "$0")"
cd "$here"/..

EXAMPLES=(
  distributor
  event_stream
  hello_clockwork
  investments
  payments
  pyth_feed
  serum_crank
)

for ex in "${EXAMPLES[@]}"; do
  cd "$ex"
  anchor build
  cd -
done