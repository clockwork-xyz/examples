import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TokenTransfer } from "../target/types/token_transfer";

describe("token_transfer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TokenTransfer as Program<TokenTransfer>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
