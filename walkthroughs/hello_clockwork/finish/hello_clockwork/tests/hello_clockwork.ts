import {expect} from "chai";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {HelloClockwork} from "../target/types/hello_clockwork";

// 👇 The new import
import { getThreadAddress, createThread } from "@clockwork-xyz/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.HelloClockwork as Program<HelloClockwork>;


const buildHelloInstruction = async () => {
    return program.methods
        .helloIx()
        .accounts({})
        .instruction();
}

describe("hello_clockwork", () => {
    it("It creates a Thread!", async () => {
        // 1️⃣ Prepare an instruction to feed to the Thread
        const targetIx = await buildHelloInstruction()

        // 2️⃣ Define a trigger for the Thread to execute
        const trigger = {
            cron: {
                schedule: "*/10 * * * * * *",
                skippable: true,
            },
        }

        // Accounts
        const threadLabel = "hello_clockwork_feb_15_23:26";
        const threadAuthority = provider.publicKey;
        const payer = provider.publicKey;
        const threadAddress = getThreadAddress(threadAuthority, threadLabel);

        // 3️⃣ Create Thread
        const createThreadIx = createThread({
            instruction: targetIx,
            trigger: trigger,
            threadName: threadLabel,
            threadAuthority: threadAuthority
        }, provider);

        try {
            const tx = await createThreadIx;
            print_address("🤖 Program", program.programId.toString());
            print_thread_address("🧵 Thread", threadAddress);
            print_tx("✍️ Tx", tx);
        } catch (e) {
            // ❌
            // 'Program log: Instruction: ThreadCreate',
            //     'Program 11111111111111111111111111111111 invoke [2]',
            //     'Allocate: account Address { address: ..., base: None } already in use'
            //
            // -> If you encounter this error, the thread address you are trying to assign is already in use,
            //    you can just change the threadLabel, to generate a new address.
            // -> OR update the thread with a ThreadUpdate instruction (more on this in another guide)
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
