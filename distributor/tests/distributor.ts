import { spawn } from "child_process";
import { assert, expect } from "chai";
import * as anchor from "@project-serum/anchor";
import { AnchorProvider, Program } from "@project-serum/anchor";
import {
    Keypair, PublicKey, Signer, SystemProgram,
    LAMPORTS_PER_SOL, SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
    createMint,
    getAssociatedTokenAddress,
    TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getMint, getAccount,
} from "@solana/spl-token";
import { Distributor } from "../target/types/distributor";
import { print_address, print_thread, verifyAmount, waitForThreadExec } from "../../utils/helpers";

// 👇 The new import
import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const clockworkProvider = new ClockworkProvider(provider.wallet, provider.connection);
// 👇 will get fixed in future version of ClockworkProvider
clockworkProvider.threadProgram.provider.connection.opts = AnchorProvider.defaultOptions();
const program = anchor.workspace.Distributor as Program<Distributor>;

const THREAD_PROGRAM_ID = new PublicKey("CLoCKyJ6DXBJqqu2VWx9RLbgnwwR6BMHHuyasVmfMzBh");

describe("distributor", () => {
    print_address("🤖 Program", program.programId.toString());

    it("It distributes tokens!", async () => {
        const [authority, mint, bob, bobAta, charlie, charlieAta] = await prepareAccounts();

        const [distributor] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("distributor"), // 👈 make sure it matches on the prog side
                mint.toBuffer(),
                authority.publicKey.toBuffer(),
            ],
            program.programId
        );

        print_address("mint", mint);
        print_address("bob's token account", bobAta);
        print_address("charlie's token account", charlieAta);

        try {
            const amount = BigInt(1_000);

            // Create Distributor
            await createDistributor(
                authority,
                distributor,
                amount,
                mint,
                bob,
                bobAta,
            )

            // Create Distributor Thread
            const thread = await createDistributorThread(
                authority,
                distributor,
                mint,
                bob,
                bobAta,
            );

            // Verifying that bob has received the tokens
            console.log(`Verifying that Thread distributed ${amount} tokens to Bob...`);
            await waitForThreadExec(clockworkProvider, thread);
            const bobAmount = await verifyAmount(provider.connection, bobAta, amount);
            console.log(`Bob has received ${bobAmount} tokens`);

            // Verifying that we can change the distributor information
            const newAmount = BigInt(2_000);
            console.log(`Asking Thread to mint to Charlie every 10s (instead of Bob) at ${newAmount} tokens`);
            await updateDistributor(authority, distributor, thread, newAmount, mint, charlie);
            const mintAmount = (await program.account.distributor.fetch(distributor)).mintAmount;
            assert.equal(mintAmount.toString(), newAmount.toString());

            // Verifying that Charlie has received the tokens
            console.log(`Verifying that Thread distributed ${newAmount} tokens to Charlie instead of Bob`);
            await waitForThreadExec(clockworkProvider, thread);

            const charlieAmountLOL = (await getAccount(provider.connection, charlieAta)).amount;
            console.log(`CHARLIE AMOUNT: ${charlieAmountLOL}`);

            const charlieAmount = await verifyAmount(provider.connection, charlieAta, newAmount);
            console.log(`Charlie has received ${charlieAmount} tokens`);

            const bobAmount2 = await verifyAmount(provider.connection, bobAta, bobAmount);
            console.log(`Bob is not receiving tokens anymore and holds ${bobAmount2} `);
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
    });
});

const prepareAccounts = async (): Promise<[Signer, PublicKey, PublicKey, PublicKey, PublicKey, PublicKey]> => {
    const authority = provider.wallet.payer;
    const mint = await createMint(
        provider.connection,
        authority,
        authority.publicKey,
        null,
        9
    );
    const bob = Keypair.generate().publicKey;
    const bobAta = await getAssociatedTokenAddress(mint, bob);
    const charlie = Keypair.generate().publicKey;
    const charlieAta = await getAssociatedTokenAddress(mint, charlie);
    return [authority, mint, bob, bobAta, charlie, charlieAta];
}

const createDistributor = async (
    authority: Signer,
    distributor: PublicKey,
    amount: bigint,
    mint: PublicKey,
    recipient: PublicKey,
    recipientAta: PublicKey,
) => {
    await program.methods
        .create(new anchor.BN(amount.toString()))
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            authority: authority.publicKey,
            mint: mint,
            distributor: distributor,
            recipient: recipient,
            recipientTokenAccount: recipientAta
        })
        .rpc();
}

const createDistributorThread = async (
    authority: Signer,
    distributor: PublicKey,
    mint: PublicKey,
    recipient: PublicKey,
    recipientAta: PublicKey,
) => {
    // const threadId = "distributor"; // 👈 make sure it matches on the prog side
    // For debug: use a fix thread name such as the above, when your code works!
    const date = new Date();
    const threadId = "distributor_" + date.toLocaleDateString() + "-" + date.getHours() + ":" + date.getMinutes();
    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    const threadAuthority = authority.publicKey;
    const [threadAddress] = clockworkProvider.getThreadPDA(threadAuthority, threadId);

    // https://docs.rs/clockwork-utils/latest/clockwork_utils/static.PAYER_PUBKEY.html
    const payer = PAYER_PUBKEY;

    const targetIx = await program.methods
        .distribute()
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            payer: payer,
            mint: mint,
            distributor: distributor,
            distributorThread: threadAddress,
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

    // 💰 Top-up the thread with this amount of SOL to spend
    // Each tx ran by your thread will cost 1000 LAMPORTS
    const threadSOLBudget = LAMPORTS_PER_SOL;
    await clockworkProvider.threadCreate(
        threadAuthority,
        threadId,
        [targetIx],
        trigger,
        threadSOLBudget
    );

    await print_thread(clockworkProvider, threadAddress);
    return threadAddress;
}

const updateDistributor = async (
    authority: Signer,
    distributor: PublicKey,
    distributorThread: PublicKey,
    amount: bigint,
    mint: PublicKey,
    charlie: PublicKey
) => {
    const cronSchedule = "*/10 * * * * * *";
    await program.methods
        .update(charlie, new anchor.BN(amount.toString()), cronSchedule)
        .accounts({
            systemProgram: SystemProgram.programId,
            clockworkProgram: THREAD_PROGRAM_ID,
            authority: authority.publicKey,
            mint: mint,
            distributor: distributor,
            distributorThread: distributorThread,
        })
        .rpc();
}
