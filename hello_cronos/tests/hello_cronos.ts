import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { HelloCronos } from "../target/types/hello_cronos";

describe("hello_cronos", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.HelloCronos as Program<HelloCronos>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
