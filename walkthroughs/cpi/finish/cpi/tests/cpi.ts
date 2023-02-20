import {expect} from "chai";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Cpi } from "../target/types/cpi";
import {PublicKey, SystemProgram} from "@solana/web3.js";
// 👇 The new import
import {getThreadAddress, CLOCKWORK_THREAD_PROGRAM_ID} from "@clockwork-xyz/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Cpi as Program<Cpi>;


describe("cpi", () => {
  it("It creates a Thread!", async () => {
    // 0️⃣ Accounts
    const threadLabel = "cpi_21-0105";
    const [threadAuthority] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("authority")], // 👈 make sure it matches on the prog side
        program.programId
    );
    const threadAddress = getThreadAddress(threadAuthority, threadLabel);

    // 1️⃣ Ask your Driver Program to do a CPI to create a Thread
    try {
      const tx = await program.methods.createThread(threadLabel)
          .accounts({
            systemProgram: SystemProgram.programId,
            clockworkProgram: CLOCKWORK_THREAD_PROGRAM_ID,
            payer: provider.publicKey,
            thread: threadAddress,
            threadAuthority: threadAuthority,
          })
          .rpc();
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


      // ❌
      // 'Program log: AnchorError caused by account: thread. Error Code: AccountNotSystemOwned.
      // Error Number: 3011. Error Message: The given account is not owned by the system program.',
      //
      // -> Same as the above, actually
      //    What's happening is that, the account is now owned by the ThreadProgram,
      //    and thus the Account has been successfully created.
      //    The account owner is now the ThreadProgram, but in your program you are expecting a SystemAccount
      // -> What to do? -> (Same as the first error) for your tests it's fine to just use a new address.
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
