import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { EventStream } from "../target/types/event_stream";

describe("event_stream", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.EventStream as Program<EventStream>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
