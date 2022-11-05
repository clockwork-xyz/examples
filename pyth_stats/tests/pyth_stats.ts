import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PythStats } from "../target/types/pyth_stats";

describe("pyth_stats", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PythStats as Program<PythStats>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
