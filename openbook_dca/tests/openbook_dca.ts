import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OpenbookDca } from "../target/types/openbook_dca";

describe("openbook_dca", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OpenbookDca as Program<OpenbookDca>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
