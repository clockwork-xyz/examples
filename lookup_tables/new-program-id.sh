#!/usr/bin/env bash

set -e

if [ $# -eq 0 ] || [ "$1" != "localnet" ] && [ "$1" != "devnet" ]; then
  echo "usage: $0 <localnet|devnet>"
  exit 1
fi

crate_name=$(basename "$PWD")
crate_name=$(sed -E "s/^([0-9]+)-//g" <<< "$crate_name")
network=$1

replace_in_file() {
  if [ "$(uname)" == "Darwin" ]; then
    sed -i '' -e "$1" "$2"
  else
    sed -i'' -e "$1" "$2"
  fi
}

# Get pubkey addresses
# Delete old program ID if -f is passed
if [ $# -eq 2 ] && [ "$2" == "-f" ]; then
  echo "Deleting old program id"
  rm -rf "target/deploy/$crate_name-keypair.json"
fi
anchor build

program_id=$(solana address -k "target/deploy/$crate_name-keypair.json")
echo "project_name: $crate_name"
echo "program_id: $program_id"

# Update program IDs
replace_in_file 's/^declare_id!(".*");/declare_id!("'${program_id}'");/g' "programs/$crate_name/src/lib.rs"
replace_in_file 's/^'${crate_name}' = ".*"/'${crate_name}' = "'${program_id}'"/g' Anchor.toml

# Rebuild with new program ID
anchor build

deploy_devnet() {
  solana config set --url devnet
  replace_in_file 's/^cluster = ".*"/cluster = "'${network}'"/g' Anchor.toml
  anchor deploy
}

deploy_localnet() {
  solana config set --url "http://localhost:8899"
  replace_in_file 's/^cluster = ".*"/cluster = "'${network}'"/g' Anchor.toml
  anchor deploy
}

# Deploy
if [ "$1" = "devnet" ]; then
  deploy_devnet
else
  deploy_localnet
fi

