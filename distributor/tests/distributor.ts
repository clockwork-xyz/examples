import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Distributor } from "../target/types/distributor";

describe("distributor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Distributor as Program<Distributor>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
