import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OrcaDca } from "../target/types/orca_dca";

describe("orca_dca", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OrcaDca as Program<OrcaDca>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
