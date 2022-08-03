import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HelloClockwork } from "../target/types/hello_clockwork";

describe("hello_clockwork", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.HelloClockwork as Program<HelloClockwork>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
