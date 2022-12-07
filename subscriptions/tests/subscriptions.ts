import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Subscriptions } from "../target/types/subscriptions";

describe("subscriptions", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Subscriptions as Program<Subscriptions>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
