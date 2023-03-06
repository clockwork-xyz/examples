import { spawn } from "child_process";
import { expect } from "chai";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HelloClockwork } from "../target/types/hello_clockwork";

// 0️⃣  Import the Clockwork SDK.
import { ClockworkProvider } from "@clockwork-xyz/sdk";

const print_address = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`);
}

const print_tx = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}

describe("hello_clockwork", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet;
  const program = anchor.workspace.HelloClockwork as Program<HelloClockwork>;
  const clockworkProvider = new ClockworkProvider(wallet, provider.connection);

  print_address("🔗 HelloClockwork program", program.programId.toString());

  it("It says hello", async () => {
    const tx = await program.methods.hello("world").rpc();
    print_tx("🖊️  Hello", tx);
  });

  it("It runs every 10 seconds", async () => {
    // 1️⃣  Prepare an instruction to be automated.
    const targetIx = await program.methods.hello("world").accounts({}).instruction();

    // 2️⃣  Define a trigger condition for the thread.
    const trigger = {
      cron: {
        schedule: "*/10 * * * * * *",
        skippable: true,
      },
    }

    // 3️⃣  Create the thread.
    try {
      const threadId = "test-" + new Date().getTime() / 1000;
      const tx = await clockworkProvider.threadCreate(
        wallet.publicKey, // authority
        threadId,               // id
        [targetIx],             // instructions to execute
        trigger,                // trigger condition
        anchor.web3.LAMPORTS_PER_SOL, // pre-fund amount
      );
      const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(wallet.publicKey, threadId)
      const threadAccount = await clockworkProvider.getThreadAccount(threadAddress);

      console.log("\nThread: ", threadAccount, "\n");
      print_address("🧵 Thread", threadAddress);
      print_tx("🖊️ ThreadCreate", tx);
      console.log("\n");

      const cmd = spawn("solana", ["logs", "-u", "devnet", program.programId.toString()]);
      cmd.stdout.on("data", data => {
          console.log(`Program Logs: ${data}`);
      });
    } catch (e) {
      // ❌
      // 'Program log: Instruction: ThreadCreate',
      // 'Program 11111111111111111111111111111111 invoke [2]',
      // 'Allocate: account Address { address: ..., base: None } already in use'
      //
      // -> If you encounter this error, the thread address you are trying to use is already in use.
      //    You can change the threadId, to generate a new account address.
      // -> OR update the thread with a ThreadUpdate instruction (more on this in future guide)
      console.error(e);
      expect.fail(e);
    }
  });
});

