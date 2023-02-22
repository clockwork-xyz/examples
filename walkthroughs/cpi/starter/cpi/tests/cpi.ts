import {expect} from "chai";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Cpi } from "../target/types/cpi";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Cpi as Program<Cpi>;


describe("cpi", () => {
    it("It logs hello", async () => {
        const helloIx = program.methods.helloIx();

        try {
            const tx = await helloIx.rpc();
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
    console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`)
}

const print_tx = (label, address) => {
    console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`)
}
