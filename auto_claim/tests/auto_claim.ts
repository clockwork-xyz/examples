import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AutoClaim } from "../target/types/auto_claim";

describe("auto_claim", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AutoClaim as Program<AutoClaim>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
