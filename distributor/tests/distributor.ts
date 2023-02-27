import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor./";
import {Distributor} from "../target/types/distributor";
import {
    Keypair,
    LAMPORTS_PER_SOL,
    SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
    createMint,
    getAssociatedTokenAddress,
    TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getMint, getAccount,
} from "@solana/spl-token";

// 👇 The new import
import {getThreadAddress, CLOCKWORK_THREAD_PROGRAM_ID, createThread} from "@clockwork-xyz/sdk";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import {assert, expect} from "chai";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Distributor as Program<Distributor>;


describe("distributor", () => {
    it("It distributes tokens!", async () => {
        const [mint, bob, bobAta, charlie, charlieAta] = await prepareAccounts();

        const [distributor] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("distributor"), // 👈 make sure it matches on the prog side
                mint.toBuffer(),
                provider.publicKey.toBuffer(),
            ],
            program.programId
        );
        console.log("program logs: solana logs -u devnet | grep " + program.programId.toString() + "\n\n");
        print_address("mint", mint);
        print_address("bob's token account", bobAta);
        print_address("charlie's token account", charlieAta);

        // const threadName = "distributor"; // 👈 make sure it matches on the prog side
        // For debug: use a fix thread name such as the above, when your code works!
        const date = new Date();
        const threadName = "distributor_" + date.toLocaleDateString() + "-" + date.getHours() + ":" + date.getMinutes();
        // Security:
        // Note that we are using your default Solana paper keypair as the thread authority.
        // Feel free to use whichever authority is appropriate for your use case.
        const threadAuthority = provider.publicKey;
        const threadAddress = getThreadAddress(threadAuthority, threadName);

        // Top up the thread account with some SOL
        await provider.connection.requestAirdrop(threadAddress, LAMPORTS_PER_SOL);

        try {
            // Create Distributor
            await createDistributor(
                distributor,
                mint,
                bob,
                bobAta,
            )

            // Create Distributor Thread
            await createDistributorThread(
                distributor,
                mint,
                bob,
                bobAta,
                threadName,
                threadAddress,
                threadAuthority,
            );
            print_address("🤖 Program", program.programId.toString());
            print_thread_address("🧵 Thread", threadAddress);
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

        // Verifying that bob has received the tokens
        console.log("Verifying that Thread distributed payment to Bob...");
        const bobPreAmount = (await getAccount(provider.connection, bobAta)).amount;
        await sleep(15);
        const bobPostAmount = (await getAccount(provider.connection, bobAta)).amount;
        assert.isAtLeast(Number(bobPostAmount), Number(bobPreAmount + BigInt(100_000_000)), "Bob hasn't received" +
            " the distribution");

        // Verifying that we can change the distributor information
        console.log("Asking Thread to pay Charlie every 15s at 200_000_000 tokens");
        await updateDistributor(distributor, threadAddress, mint, charlie);
        const distrib = await program.account.distributor.fetch(distributor);
        assert.deepEqual(distrib.recipientTokenAccount, charlieAta, "distributor's recipient" +
            " unchanged");
        assert.deepEqual(distrib.mint, mint, "distributor's mint unchanged");

        console.log("Verifying that Thread distributed payment to Charlie...");
        let charliePreAmount = BigInt(0);
        try {
            charliePreAmount = (await getAccount(provider.connection, charlieAta)).amount;
        } catch (e) {
            console.log("charlie's ata not created yet, minting hasn't occured yet");
        }
        await sleep(20);
        const charliePostAmount = (await getAccount(provider.connection, charlieAta)).amount;
        assert.isAtLeast(Number(charliePostAmount), Number(charliePreAmount + BigInt(200_000_000)), "Charlie hasn't received the" +
            " distribution");
    });
});

const prepareAccounts = async (): Promise<[PublicKey, PublicKey, PublicKey, PublicKey, PublicKey]> => {
    const mint = await createMint(
        provider.connection,
        provider.wallet.payer,
        provider.publicKey,
        null,
        9 // decimals
    );
    const bob = Keypair.generate().publicKey;
    const bobAta = await getAssociatedTokenAddress(mint, bob);
    const charlie = Keypair.generate().publicKey;
    const charlieAta = await getAssociatedTokenAddress(mint, charlie);
    return [mint, bob, bobAta, charlie, charlieAta];
}

const createDistributor = async (distributor: PublicKey,
                                 mint: PublicKey,
                                 recipient: PublicKey,
                                 recipientAta: PublicKey,
) => {
    const createDistributorTx = await program.methods
        .create(new anchor.BN(100_000_000))
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            clockwork: CLOCKWORK_THREAD_PROGRAM_ID,
            authority: provider.publicKey,
            mint: mint,
            distributor: distributor,
            recipient: recipient,
            recipientTokenAccount: recipientAta
        })
        .rpc();
    print_tx("✍️ createDistributorTx", createDistributorTx);
    return createDistributorTx;

}

const createDistributorThread = async (distributor: PublicKey,
                                       mint: PublicKey,
                                       recipient: PublicKey,
                                       recipientAta: PublicKey,
                                       threadName: string,
                                       thread: PublicKey,
                                       threadAuthority: PublicKey,
) => {
    // https://docs.rs/clockwork-utils/latest/clockwork_utils/static.PAYER_PUBKEY.html
    const PAYER_PUBKEY = new PublicKey("C1ockworkPayer11111111111111111111111111111");

    const targetIx = await program.methods
        .distribute()
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            payer: PAYER_PUBKEY,
            mint: mint,
            distributor: distributor,
            distributorThread: thread,
            recipient: recipient,
            recipientTokenAccount: recipientAta,
        })
        .instruction();

    const trigger = {
        cron: {
            schedule: "*/10 * * * * * *",
            skippable: true,
        },
    }

    const r = await createThread({
        instruction: targetIx,
        trigger: trigger,
        threadName: threadName,
        threadAuthority: threadAuthority
    }, provider);
    console.log(r.thread);

}

const updateDistributor = async (distributor: PublicKey,
                                 distributorThread: PublicKey,
                                 mint: PublicKey,
                                 charlie: PublicKey
) => {
    const cronSchedule = "*/15 * * * * * *";
    const updateDistributorTx = await program.methods
        .update(charlie, new anchor.BN(200_000_000), cronSchedule)
        .accounts({
            systemProgram: SystemProgram.programId,
            clockworkProgram: CLOCKWORK_THREAD_PROGRAM_ID,
            authority: provider.publicKey,
            mint: mint,
            distributor: distributor,
            distributorThread: distributorThread,
        })
        .rpc();
    print_tx("✍️ updateDistributorTx", updateDistributorTx);
    return updateDistributorTx;

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
