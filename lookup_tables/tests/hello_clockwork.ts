import { expect } from "chai";
import * as web3 from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HelloClockwork } from "../target/types/hello_clockwork";
import { print_address, print_thread, print_tx, stream_program_logs } from "@utils";

// 0️⃣  Import the Clockwork SDK.
import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";

describe("hello_clockwork", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet;
  const program = anchor.workspace.HelloClockwork as Program<HelloClockwork>;
  const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);

  print_address("🔗 HelloClockwork program", program.programId.toString());

  it("It says hello", async () => {
    const tx = await program.methods.hello("world").rpc();
    print_tx("🖊️  Hello", tx);
  });

  it("It runs every 10 seconds", async () => {
    // 1️⃣  Prepare an instruction to be automated.
    const targetIx = new web3.TransactionInstruction({
      keys: [{ pubkey: PAYER_PUBKEY, isSigner: true, isWritable: true }],
      data: Buffer.from(JSON.stringify("TEST CLOCK2"), 'utf-8'),
      programId: new web3.PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
  })

    // 2️⃣  Define a trigger condition for the thread.
    const trigger = {
      cron: {
        schedule: "*/10 * * * * * *",
        skippable: true,
      },
    }

    // 3️⃣  Create the thread.
    try {
      const threadId = "hello_" + new Date().getTime() / 1000;
      const ix = await clockworkProvider.threadCreate(
        wallet.publicKey, // authority
        threadId,               // id
        [targetIx],             // instructions to execute
        trigger,                // trigger condition
        anchor.web3.LAMPORTS_PER_SOL, // pre-fund amount
      );
      const tx = new anchor.web3.Transaction().add(ix);
      const signature = await clockworkProvider.anchorProvider.sendAndConfirm(tx);

      const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(wallet.publicKey, threadId)
      await print_thread(clockworkProvider, threadAddress);
      stream_program_logs(program.programId);
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

