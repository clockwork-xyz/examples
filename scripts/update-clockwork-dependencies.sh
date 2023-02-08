#!/usr/bin/env bash

set -e

here="$(dirname "$0")"
cd "$here"/..


# Get new Clockwork version
current_version=$(cat CLOCKWORK_SDK_VERSION)
echo "Current Clockwork version: $current_version"
read -r -p "    New Clockwork version: " new_version
new_version=${new_version//[v]/''}


# # Get new Anchor version
# current_anchor_version=$(cat ANCHOR_VERSION)
# echo "Current Anchor version: $current_anchor_version"
# read -r -p "    New Anchor version (or enter if unchanged): " new_anchor_version
# new_anchor_version="${new_anchor_version:-$current_anchor_version}"


# Get new Solana version
current_solana_version=$(cat SOLANA_VERSION)
echo "Current Solana version: $current_solana_version"
read -r -p "    New Solana version (or enter if unchanged): " new_solana_version
new_solana_version="${new_solana_version:-$current_solana_version}"


# # Update the root Cargo.tomls with patches (eventually remove this once we can deploy to crates normally)
# echo "Updating patched versions"
# # ROOT_TOMLS=$(find . -maxdepth 2 -type f -iname "Cargo.toml")
# declare ROOT_TOMLS=()
# while IFS='' read -r line; do ROOT_TOMLS+=("$line"); done < <(find . -maxdepth 2 -type f -iname "Cargo.toml"  -not -path './pyth_feed/*')
# for toml in "${ROOT_TOMLS[@]}"; do
#   sed -i '' -e 's/^anchor-lang =.*/anchor-lang = { git = "https:\/\/github.com\/clockwork-xyz\/anchor", branch = "'${new_anchor_version}'" }/g' "$toml"
#   sed -i '' -e 's/^anchor-spl =.*/anchor-spl = { git = "https:\/\/github.com\/clockwork-xyz\/anchor", branch = "'${new_anchor_version}'" }/g' "$toml"
#   sed -i '' -e 's/^clockwork-sdk =.*/clockwork-sdk = { git = "https:\/\/github.com\/clockwork-xyz\/clockwork", tag = "v'${new_version}'" }/g' "$toml"
# done


# Update Clients and Programs Cargo.tomls
echo "Updating clockwork-sdk" 
declare TOMLS=()
while IFS='' read -r line; do TOMLS+=("$line"); done < <(find . -mindepth 3 -type f -iname "Cargo.toml")
for toml in "${TOMLS[@]}"; do
  sed -E -i '' -e "s:(solana-sdk = \")(=?)([0-9]+\.[0-9]+)\".*:\1\2${new_solana_version}\":" "$toml"
  sed -E -i '' -e "s:(solana-sdk = \{ version = \")(=?)[0-9]+\.[0-9]+\.[0-9]+(\".*):\1\2${new_solana_version}\3:" "$toml"
  sed -E -i '' -e "s:(solana-client = \")(=?)([0-9]+\.[0-9]+)\".*:\1\2${new_solana_version}\":" "$toml"
  sed -E -i '' -e "s:(solana-client = \{ version = \")(=?)[0-9]+\.[0-9]+\.[0-9]+(\".*):\1\2${new_solana_version}\3:" "$toml"
  sed -E -i '' -e "s:(clockwork-sdk = \")(=?)([0-9]+\.[0-9]+)\".*:\1\2${new_version}\":" "$toml"
  sed -E -i '' -e "s:(clockwork-sdk = \{ version = \")(=?)[0-9]+\.[0-9]+\.[0-9]+(\".*):\1\2${new_version}\3:" "$toml"
done


# Rebuild
source scripts/build-all.sh

# Update version
echo $new_version > CLOCKWORK_SDK_VERSION
#echo $new_anchor_version > ANCHOR_VERSION
echo $new_solana_version > SOLANA_VERSION
