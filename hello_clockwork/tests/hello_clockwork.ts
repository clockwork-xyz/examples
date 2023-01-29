import { expect } from "chai";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HelloClockwork } from "../target/types/hello_clockwork";

// 👇 The new import
import { getThreadProgram } from "@clockwork-xyz/sdk";
import { getThreadAddress } from "@clockwork-xyz/sdk/lib/pdas";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.HelloClockwork as Program<HelloClockwork>;


const buildHelloInstruction = async (name: string, thread: PublicKey) => {
  return program.methods
    .helloWorld(name)
    .accounts({ helloThread: thread })
    .instruction();
}

describe("hello_clockwork", () => {
  it("It creates a Thread!", async () => {
    // Accounts
    const threadLabel = "hello_clockwork_feb_15_23:28";
    const threadAuthority = provider.publicKey;
    const payer = provider.publicKey;
    const threadAddress = getThreadAddress(threadAuthority, threadLabel);

    // 1️⃣ Prepare an instruction to feed to the Thread
    const targetIx = await buildHelloInstruction("Chronos", threadAddress)

    // 2️⃣ Define a trigger for the Thread to execute
    const trigger = {
      cron: {
        schedule: "*/10 * * * * * *",
        skippable: true,
      },
    }

    // 3️⃣ Create Thread
    const threadProgram = getThreadProgram(provider, "1.4.2");
    const createThreadIx = threadProgram.methods
      .threadCreate(
        threadLabel,
        {
          programId: targetIx.programId,
          accounts: [
            { pubkey: threadAddress, isSigner: false, isWritable: true }
          ],
          data: targetIx.data,
        },
        trigger,
      )
      .accounts({
        authority: threadAuthority,
        payer: payer,
        thread: threadAddress,
        systemProgram: SystemProgram.programId,
      });

    try {
      const tx = await createThreadIx.rpc();
      print_address("🤖 Program", program.programId.toString());
      print_thread_address("🧵 Thread", threadAddress);
      print_tx("✍️ Tx", tx);
    } catch (e) {
      // ❌
      // 'Program log: Instruction: ThreadCreate',
      //     'Program 11111111111111111111111111111111 invoke [2]',
      //     'Allocate: account Address { address: ..., base: None } already in use'
      //
      // -> If you encounter this error, the thread address you are trying to assign is already in use,
      //    you can just change the threadLabel, to generate a new address.
      // -> OR update the thread with a ThreadUpdate instruction (more on this in another guide)
      console.error(e);
      expect.fail(e);
    }
  });
});

const print_address = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`);
}

const print_tx = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}

const print_thread_address = (label, address) => {
  console.log(`${label}: https://explorer.clockwork.xyz/address/${address}?network=devnet`);
}
