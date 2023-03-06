import { expect } from "chai";
import { PublicKey, SystemProgram, } from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Counter } from "../target/types/counter";
import { print_address, print_thread, print_tx, waitForThreadExec } from "../../utils/helpers";

// 0️⃣  Import the Clockwork SDK.
import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";


const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;
const program = anchor.workspace.Counter as Program<Counter>;
const clockworkProvider = new ClockworkProvider(wallet, provider.connection);


// Helpers
const getCounterAddress = () => {
    return PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("counter")], // 👈 make sure it matches on the prog side
        program.programId
    )[0];
}

const fetchCounter = async (counter) => {
    const counterAcc = await program.account.counter.fetch(counter);
    console.log("currentValue: " + counterAcc.currentValue + ", updatedAt: " + counterAcc.updatedAt);
    return counterAcc;
}


// Tests
describe("counter", () => {
    const counter = getCounterAddress();
    print_address("🤖 Counter program", program.programId.toString());

    beforeEach(async () => {
        await program.methods
            .reset()
            .accounts({
                systemProgram: SystemProgram.programId,
                payer: wallet.publicKey,
                counter: counter,
            })
            .rpc();
    })

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

    it("It increments every 10 seconds", async () => {
        // 1️⃣  Prepare an instruction to be automated.
        const targetIx = await program.methods
            .increment()
            .accounts({
                systemProgram: SystemProgram.programId,
                payer: wallet.publicKey,
                counter: counter,
            })
            .instruction();

        // 2️⃣  Define a trigger condition for the thread.
        const trigger = {
            cron: {
                schedule: "*/10 * * * * * *",
                skippable: true,
            },
        }

        // 3️⃣  Create the thread.
        const threadId = "counter-" + new Date().getTime() / 1000;
        const [threadAuthority] = PublicKey.findProgramAddressSync(
            [anchor.utils.bytes.utf8.encode("authority")], // 👈 make sure it matches on the prog side
            program.programId
        );
        const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(threadAuthority, threadId)
        try {
            await program.methods
                .createThread(Buffer.from(threadId))
                .accounts({
                    systemProgram: SystemProgram.programId,
                    clockworkProgram: clockworkProvider.threadProgram.programId,
                    payer: wallet.publicKey,
                    thread: threadAddress,
                    threadAuthority: threadAuthority,
                    counter: counter,
                })
                .rpc();

            await print_thread(clockworkProvider, threadAddress);

            console.log("Verifying that Thread increments the counter every 10s")
            for (let i = 1; i < 4; i++) {
                await waitForThreadExec(clockworkProvider, threadAddress);
                const counterAcc = await fetchCounter(counter);
                expect(counterAcc.currentValue.toString()).to.eq(i.toString());
            }
        } catch (e) {
            // ❌
            // 'Program log: Instruction: ThreadCreate',
            // 'Program 11111111111111111111111111111111 invoke [2]',
            // 'Allocate: account Address { address: ..., base: None } already in use'
            //
            // -> If you encounter this error, the thread address you are trying to use is already in use.
            //    You can change the threadId, to generate a new account address.
            // -> OR update the thread with a ThreadUpdate instruction (more on this in future guide)
            console.error(e);
            expect.fail(e);
        } finally {
            await program.methods
                .deleteThread()
                .accounts({
                    clockworkProgram: clockworkProvider.threadProgram.programId,
                    payer: wallet.publicKey,
                    thread: threadAddress,
                    threadAuthority: threadAuthority,
                })
                .rpc();
        }
    })
});
