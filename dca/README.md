# **Dollar Cost Averaging**

## Prerequisites
- Make sure you have both the [solana cli](https://docs.solana.com/cli/install-solana-cli-tools) and [anchor cli](https://project-serum.github.io/anchor/getting-started/installation.html#build-from-source-for-other-operating-systems) installed on your computer.

## Deploying DCA
- run `anchor build` in the root directory of `dca`
- run `solana address -k target/deploy/dca-keypair.json` to get your program's ID
- copy that ID and replace it with the Program ID in `id.rs`
- run `anchor build` again
- be sure to set your solana config to devnet with `solana config set --url https://api.devnet.solana.com`
- run `solana airdrop 2` a couple of times to ensure that you have enough sol to pay for this transaction
- run `solana program deploy target/deploy/dca.so` or `anchor deploy` to deploy this program on-chain

## Invoking DCA Program
- navigate to the `client` directory
- run `cargo run` 