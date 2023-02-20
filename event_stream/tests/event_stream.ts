import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {EventStream} from "../target/types/event_stream";
// 👇 The new import
import {getThreadAddress, CLOCKWORK_THREAD_PROGRAM_ID} from "@clockwork-xyz/sdk";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import {expect} from "chai";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.EventStream as Program<EventStream>;


describe("event_stream", () => {
    it("It creates a Thread!", async () => {
        const [event] = PublicKey.findProgramAddressSync(
            [anchor.utils.bytes.utf8.encode("event")], // 👈 make sure it matches on the prog side
            program.programId
        );

        await initializeThread(event);

        // Ping an account change every 10s
        // Ask program to update the account with address `event`
        for (let i = 0; i < 9; i++) {
            await sleep(10);
            await ping(event);
        }
    });
});

const initializeThread = async (event: PublicKey) => {
    // 0️⃣ Accounts
    const threadLabel = "eventstream_21-0151";
    const [threadAuthority] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("authority")], // 👈 make sure it matches on the prog side
        program.programId
    );
    const threadAddress = getThreadAddress(threadAuthority, threadLabel);

    console.log("event:" + event.toString());
    console.log("threadLabel:" + threadLabel.toString());
    console.log("threadAuthority:" + threadAuthority.toString());
    console.log("threadAddress:" + threadAddress.toString());

    // 1️⃣ Ask your Program to do a CPI to create a Thread
    try {
        const tx = await program.methods.initialize(threadLabel)
            .accounts({
                systemProgram: SystemProgram.programId,
                clockwork: CLOCKWORK_THREAD_PROGRAM_ID,
                signer: provider.publicKey,
                authority: threadAuthority,
                eventThread: threadAddress,
                event: event,
            })
            .rpc();
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


        // ❌
        // 'Program log: AnchorError caused by account: thread. Error Code: AccountNotSystemOwned.
        // Error Number: 3011. Error Message: The given account is not owned by the system program.',
        //
        // -> Same as the above, actually
        //    What's happening is that, the account is now owned by the ThreadProgram,
        //    and thus the Account has been successfully created.
        //    The account owner is now the ThreadProgram, but in your program you are expecting a SystemAccount
        // -> What to do? -> (Same as the first error) for your tests it's fine to just use a new address.
        console.error(e);
        expect.fail(e);
    }
}

const ping = async (event: PublicKey) => {
    try {
        const tx = await program.methods.ping()
            .accounts({event: event})
            .rpc()
        print_tx("✍️ Ping Tx", tx);
    } catch (e) {
        console.error(e);
        expect.fail(e);
    }
}

function sleep(seconds) {
    return new Promise((resolve) => {
        setTimeout(resolve, 1000 * seconds);
    });
}

const print_address = (label, address) => {
    console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`);
}

const print_tx = (label, address) => {
    console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}

const print_thread_address = (label, address) => {
    console.log(`${label}: https://explorer.clockwork.xyz/address/${address}?network=devnet`);
}
