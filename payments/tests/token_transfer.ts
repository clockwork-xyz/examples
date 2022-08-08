import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PaymentsProgram } from "../target/types/payments_program";

describe("payments_program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PaymentsProgram as Program<PaymentsProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
