import { expect } from "chai";
import {
    Keypair, PublicKey, Signer, SystemProgram,
    LAMPORTS_PER_SOL, SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Counter } from "../target/types/counter";


const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;
const program = anchor.workspace.Counter as Program<Counter>;


// Helpers
const getCounterAddress = () => {
    return PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("counter")], // 👈 make sure it matches on the prog side
        program.programId
    )[0];
}

const fetchCounter = async (counter) => {
    const counterAcc = await program.account.counter.fetch(counter);
    console.log("currentValue: " + counterAcc.currentValue);
    console.log("updatedAt: " +  counterAcc.updatedAt);
    return counterAcc;
}

const print_tx = (label, address) => {
    console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}


// Tests
describe("counter", () => {
    const counter = getCounterAddress();

    it("It increments the counter", async () => {
        const tx = await program.methods
            .increment()
            .accounts({
                systemProgram: SystemProgram.programId,
                payer: wallet.publicKey,
                counter: counter,
            })
            .rpc();

        const counterAcc = await fetchCounter(counter);
        expect(counterAcc.currentValue.toString()).to.eq("1");
        expect(counterAcc.updatedAt.toString()).to.not.eq("0");
    });

    it ("It increments every 10 seconds", async () => {
        
    })
});