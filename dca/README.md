# **DCA**

## Prerequisites
- Make sure you have both the [solana cli](https://docs.solana.com/cli/install-solana-cli-tools) and [anchor cli](https://project-serum.github.io/anchor/getting-started/installation.html#build-from-source-for-other-operating-systems) installed on your computer.

## Deploying and invoking DCA Program
- run `anchor build` in the root directory of `dca`
- run `solana address -k target/deploy/dca-keypair.json` to get your program's ID
- copy that ID and replace it with the Program ID in `id.rs` and `Anchor.toml`
- run `anchor build` again
- be sure to set your solana config to localnet with `solana config set --url http://localhost:8899`
- clone [cronos repo](https://github.com/cronos-so/cronos)
- in the cronos repo add these two lines to the `localnet.sh` script in the to deploy both the dca and serum_dex programs to your local validator
  ```bash
    --bpf-program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin <path to dca/dex/serum_dex.so> \
    --bpf-program <path to target/deploy/dca-keypair.json> <path to target/deploy/dca.so> \
  ```
- run `./scripts/localnet.sh`
- in a second terminal while also in the cronos repo run `./scripts/tail-logs.sh`
- lastly, in a third terminal run `./scripts/initialize-programs.sh && ./scripts/stake-localnet.sh`
- back in the examples repo and replace `dex_program_id` variable with `let dex_program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();` in `client/main.rs`
- navigate the the `client` directory and run `cargo run`

<!-- ### Devnet
- run `anchor build` in the root directory of `dca`
- run `solana address -k target/deploy/dca-keypair.json` to get your program's ID
- copy that ID and replace it with the Program ID in `id.rs` and the `Anchor.toml` files
- also change `cluster = "devnet"` in `Anchor.toml`
- run `anchor build` again
- be sure to set your solana config to localnet with `solana config set --url https://api.devnet.solana.com`
- run `solana airdrop 2` a couple of times to ensure that you have enough sol to pay for this transaction
- run `solana program deploy target/deploy/dca.so` to deploy this program on-chain
- navigate the the `client` directory and run `cargo run` -->