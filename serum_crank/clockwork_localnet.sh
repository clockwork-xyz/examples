#!/usr/bin/env bash

rm -rf test-ledger
anchor build -- --features localnet

clockwork localnet \
    --bpf-program ~/examples/serum_crank/dex/serum_dex-keypair.json \
    --bpf-program ~/examples/serum_crank/dex/serum_dex.so \
    --bpf-program ~/examples/serum_crank/target/deploy/serum_crank-keypair.json \
    --bpf-program ~/examples/serum_crank/target/deploy/serum_crank.so

