# **Event Stream Program**

## Devnet
**1. Program Side - Deploying the Handler Program**
We start by defining an instruction to execute, that is the __"WHAT"__:
- We have already prepared a handler.
- You just have to deploy it using `./deploy.sh devnet` _(nothing fancy, the usual program ids and network switch)_.

**2. Client Side - Creating a Thread**
Time to switch perspective, we need to do some work off-chain now, we will create a Thread, that's the __"HOW"__:
- Check the `tests` folder, we are using anchor tests as a client.
- Run `anchor tests`, this will create a __Thread__ that listens for a certain account and print logs whenever the
  account is updated.

## How do I know if it works?
Let's see how we can observe our newly created Thread:
- The prepared client, will also print the Solana Explorer url
```bash
initialize tx: ✅ https://explorer.solana.com/tx/...
ping tx: ✅ https://explorer.solana.com/tx/...
...
```
- If you have the Clockwork Cli installed, you can use the `clockwork` command
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
We start by defining an instruction to execute, that is the __"WHAT"__:
- We have already prepared a handler.
- You just have to deploy it using `./deploy.sh localnet` _(nothing fancy, the usual program ids and network switch)_.

**2. Client Side - Creating a Thread**
Time to switch perspective, we need to do some work off-chain now, we will create a Thread, that's the __"HOW"__:
- Check the `tests` folder, we are using anchor tests as a client.
- Run `anchor tests`, this will create a __Thread__ that listens for a certain account and print logs whenever the 
  account is updated.

## Common Errors
Please refer to the [FAQ](https://github.com/clockwork-xyz/docs/blob/main/FAQ.md#common-errors).

## Questions
Come build with us and ask us questions [Discord](https://discord.gg/epHsTsnUre)!
