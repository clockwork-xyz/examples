<div align="center">
  <img height="170" src="https://raw.githubusercontent.com/openbook-dex/resources/main/brand/OpenBook-Logomark.svg" />

  <h1><a href="https://github.com/openbook-dex/program">Openbook</a> Crank Program</h1>
</div>

---

## Devnet
TBD

## Localnet

The workflow is pretty similar with __devnet__ with the exception that you are running your own validator. So you need:
1. The `solana-test-validator` which should be installed by default with Solana.
2. Install and run the Clockwork Engine _(geyser plugin)_ with the `solana-test-validator`.


**0. Program Side - Preparing the Handler Program**

We start by defining an instruction to execute, that is the __"WHAT"__:
- We have already prepared a handler in `./programs/`.

**1. Validator Setup - Run the validator with Clockwork**
- Build and install the [Clockwork Engine](https://github.com/clockwork-xyz/clockwork#local-development)
- Run `./clockwork_localnet.sh` to start a local validator with the Clockwork Engine + Openbook Programs + Our 
  Handler Program from 👆.

**2. Client Side - Creating a Thread**

Time to switch perspective, we need to do some work off-chain now, we will create a Thread, that's the __"HOW"__:
- Navigate to the `client` directory. _We have prepared a Rust client, but you
  can very well use any Solana client you prefer (JS, Python, etc)._
- Run `cargo run --features localnet`, this will create a __Thread__ that continuously calls your program handler, and so continously
  prints "Hello World".

## Common Errors
Please refer to the [FAQ](https://github.com/clockwork-xyz/docs/blob/main/FAQ.md#common-errors).

## Questions
Come build with us and ask us questions [Discord](https://discord.gg/epHsTsnUre)!
