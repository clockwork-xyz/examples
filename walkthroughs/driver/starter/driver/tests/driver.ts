import {expect} from "chai";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {Driver} from "../target/types/driver";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Driver as Program<Driver>;


describe("driver", () => {
  it("It creates a Thread!", async () => {
    const createThreadIx = program.methods.createThread();

    try {
      const tx = await createThreadIx.rpc();
      print_address("🤖 Program", program.programId.toString());
      // print_thread_address("🧵 Thread", threadAddress);
      print_tx("✍️ Tx", tx);
    } catch (e) {
      console.error(e);
      expect.fail(e);
    }
  });
});


const print_address = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`);
}

const print_tx = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}

const print_thread_address = (label, address) => {
  console.log(`${label}: https://explorer.clockwork.xyz/address/${address}?network=devnet`);
}
