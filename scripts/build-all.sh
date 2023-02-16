#!/usr/bin/env bash

set -e

here="$(dirname "$0")"
cd "$here"/..

EXAMPLES=(
  distributor
  event_stream
  hello_clockwork
  openbook_crank
  openbook_dca
  payments
  pyth_feed
  pyth_stats
#  subscriptions not migrated to 1.4.2 yet
)

for ex in "${EXAMPLES[@]}"; do
  cd "$ex"
  echo "building $ex"
  cargo build
  cd -
done
