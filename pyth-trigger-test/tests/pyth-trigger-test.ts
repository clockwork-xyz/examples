import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { PythTriggerTest } from "../target/types/pyth_trigger_test";
import { ClockworkProvider } from "@clockwork-xyz/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;
// const program = anchor.workspace.Counter as Program<Counter>;
  const program = anchor.workspace.PythTriggerTest as Program<PythTriggerTest>;
const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);


describe("pyth-trigger-test", () => {
  // Configure the client to use the local cluster.
  // anchor.setProvider(anchor.AnchorProvider.env());
  // const wallet = provider.wallet;
  // const program = anchor.workspace.PythTriggerTest as Program<PythTriggerTest>;
  // const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);

  it("Is initialized!", async () => {
    const threadId = "";
    const [threadAuthority] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("authority")], // 👈 make sure it matches on the prog side
        program.programId
    );
    const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(threadAuthority, threadId)

    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accounts({
        payer: wallet.publicKey,
        clockworkProgram: clockworkProvider.threadProgram.programId,
        systemProgram: SystemProgram.programId,
        threadAuthority: threadAuthority,
        thread: threadAddress,
      })
      .rpc();
    
    console.log("Your transaction signature", tx);
  });
});
