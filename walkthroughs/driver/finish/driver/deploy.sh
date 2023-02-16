#!/usr/bin/env bash

set -e

if [ $# -eq 0 ] || [ "$1" != "localnet" ] && [ "$1" != "devnet" ]; then
  echo "usage: $0 <localnet|devnet>"
  exit 1
fi

crate_name=$(basename "$PWD")
network=$1

# Get pubkey addresses
anchor build

program_id=$(solana address -k "target/deploy/$crate_name-keypair.json")
echo "project_name: $crate_name"
echo "program_id: $program_id"

# Update program IDs
sed -i '' -e 's/^declare_id!(".*");/declare_id!("'${program_id}'");/g' "programs/$crate_name/src/id.rs"
sed -i '' -e 's/^'${crate_name}' = ".*"/'${crate_name}' = "'${program_id}'"/g' Anchor.toml

# Rebuild with new program ID
anchor build

deploy_devnet() {
  solana config set --url devnet
  solana airdrop 2
  sed -i '' -e 's/^cluster = ".*"/cluster = "'${network}'"/g' Anchor.toml
  anchor deploy
}

deploy_localnet() {
  solana config set --url "http://localhost:8899"
  sed -i '' -e 's/^cluster = ".*"/cluster = "'${network}'"/g' Anchor.toml
  anchor deploy
}

# Deploy
if [ "$1" = "devnet" ]; then
  deploy_devnet
else
  deploy_localnet
fi

