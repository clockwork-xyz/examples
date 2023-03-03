# **Distributor Program (Token Minter)**

This program creates a Thread that stream token mints to a recipient.
- First, we create a distributor account and transfer the token mint authority to the distributor account.
- Then, we schedule a Thread to mint tokens using the distributor account's information.

---
## ⚠️ V2
Note the cargo patch in `Cargo.toml`
```toml
[patch.crates-io]
clockwork-sdk = { git = "https://github.com/clockwork-xyz/clockwork", tag = "v2.0.1" }
```
---

## Prerequisites
- Make sure you have both [Solana](https://docs.solana.com/cli/install-solana-cli-tools) and [Anchor](https://www.anchor-lang.com/docs/installation) installed on your computer.
- If you want to test on localnet, also clone the [clockwork engine](https://github.com/clockwork-xyz/clockwork/)
  locally to your machine.

> We recommend using devnet, as the setup is easier. You don't need a validator or the clockwork engine
> running. However, you will get an easier time looking at the logs if you have your own validator running locally
> later.

## Workflow
Basically the workflow goes like this:
1. First, we prepare and deploy our instruction (program) to execute: the __WHAT TO DO__.
2. Then, we create a new Thread to define the condition on __HOW__ to run the 👆instruction. _Do you want to run the
   above instruction based on a condition? Do you want to run the instruction continuously every x sec? Etc._

## Devnet
**1. Program Side - Deploying the Handler Program**

We start by defining an instruction to execute, that is the __"WHAT"__:
- We have already prepared a handler.
- You just have to deploy it using `./deploy.sh devnet` _(nothing fancy, the usual program ids and network switch)_.

**2. Client Side - Creating a Thread**

Time to switch perspective, we need to do some work off-chain now, we will create a Thread, that's the __"HOW"__:
- Check the `tests` folder, we are using anchor tests as a client.
- Run `anchor tests`, this will create a __Thread__ that continuously distributes tokens according to the
  distributor account.

## How do I know if it works?
Let's see how we can observe our newly created Thread:
- Run the tests!
```bash
anchor test
```
- Additionally, the tests will also print an url to inspect the Thread with the [Clockwork Explorer](https://explorer.
  clockwork.xyz/)
```bash
https://explorer.clockwork.xyz/address/...
```
- Finally, if you have the Clockwork Cli installed, you can use the `clockwork` command
```bash
clockwork thread get --address <your_thread_address> 
clockwork thread get <your_thread_label>
```

---

## Localnet

The workflow is pretty similar with __devnet__ with the exception that you are running your own validator. So you need:
1. The `solana-test-validator` which should be installed by default with Solana.
2. Install and run the Clockwork Engine _(geyser plugin)_ with the `solana-test-validator`.

**0. Validator Setup - Run the validator with Clockwork**

- Build and install the [Clockwork Engine](https://github.com/clockwork-xyz/clockwork#local-development)
- Use the `clockwork localnet` to start a local validator with the Clockwork Engine.

**1. Program Side - Deploying the Handler Program**

- Same as devnet but use `./deploy.sh localnet`

**2. Client Side - Creating a Thread**

- Same as devnet


## Common Errors
Please refer to the [FAQ](https://github.com/clockwork-xyz/docs/blob/main/FAQ.md#common-errors).

## Questions
Come build with us and ask us questions [Discord](https://discord.gg/epHsTsnUre)!
