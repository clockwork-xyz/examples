import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PythFeed } from "../target/types/pyth_feed";

describe("pyth_feed", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PythFeed as Program<PythFeed>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
