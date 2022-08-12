# **Investments Program**

## Prerequisites
- Make sure you have both the [solana cli](https://docs.solana.com/cli/install-solana-cli-tools) and [anchor cli](https://project-serum.github.io/anchor/getting-started/installation.html#build-from-source-for-other-operating-systems) installed on your computer.

## Deploying Investments Program
- run `anchor build` in the root directory of `investments`
- run `solana address -k target/deploy/investments-keypair.json` to get your program's ID
- copy that ID and replace it with the Program ID in `id.rs`
- run `anchor build` again
- be sure to set your solana config to devnet with `solana config set --url https://api.devnet.solana.com`
- run `solana airdrop 2` a couple of times to ensure that you have enough sol to pay for this transaction
- run `solana program deploy target/deploy/investments.so` or `anchor deploy` to deploy this program on-chain

## Invoking investments Program
- navigate to the `client` directory
- run `cargo run` 