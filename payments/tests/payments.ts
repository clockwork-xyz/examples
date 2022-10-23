import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Payments } from "../target/types/payments";

describe("payments", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Payments as Program<Payments>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
