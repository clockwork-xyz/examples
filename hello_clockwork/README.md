# **Hello Clockwork**

## Prerequisites
- Make sure you have both the [solana cli](https://docs.solana.com/cli/install-solana-cli-tools) and [anchor cli](https://project-serum.github.io/anchor/getting-started/installation.html#build-from-source-for-other-operating-systems) installed on your computer.
- If you want to test locally, also clone the [clockwork engine](https://github.com/clockwork-xyz/clockwork/) locally to your machine 

> We recommend using devnet, as the setup is easier. You don't have need a validator plugin and the clockwork engine running. However you will get an easier time looking at the logs if you have your own validator running locally later.

## Devnet
**1. Program Side - Deploying the Handler Program**
- We have already prepared a handler, a program that echoes 'Hello world'
- You just have to deploy it using `./deploy.sh devnet` _(nothing fancy, the usual program ids and network switch)_

**2. Client Side - Creating a Queue**
- Time to switch perspective, we need to do some work off-chain now.
- Navigate to the `client` directory. _The client is written in rust, but you can very well use any Solana client SDK you prefer_
- Run `cargo run --features devnet` _this will create a queue that continuously runs call your program handler_

## Localnet

The workflow is pretty similar with devnet with the exception that you are running your own validator. So you need to run it with the clockwork engine (geyser plugin).

**0. Validator Setup - Run the validator with Clockwork**
- Clone the [Clockwork Validator Plugin](https://github.com/clockwork-xyz/clockwork/#getting-started)
- And follow the [Getting Started](https://github.com/clockwork-xyz/clockwork/#getting-started)

**1. Program Side - Deploying the Handler Program**
- Deploy the handler program using `./deploy.sh localnet`

**2. Client Side - Creating a Queue**
- In a new terminal navigate to the `client` directory
- Run `cargo run`

## What is Happening? WIP
Let's see observe our transaction thread...
- solana queue get
- explorer

## But How Does It Work? WIP
1. First we define a handler, it's a bit like defining a function that we want to run.
2. Deploy it as a program
3. Create a queue 


# FAQ WIP
- foobar

# Questions
Let us know how we can help on [Discord](https://discord.gg/epHsTsnUre)!
