import {assert, expect} from "chai";
import * as anchor from "@project-serum/anchor";
import {Program, AnchorProvider} from "@project-serum/anchor";
import {Payments} from "../target/types/payments";
import {
    Keypair, PublicKey, Signer, SystemProgram,
    LAMPORTS_PER_SOL, SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
    createMint,
    createAssociatedTokenAccount,
    getAssociatedTokenAddress,
    mintToChecked,
    TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount,
} from "@solana/spl-token";

// 👇 The new import
import {ClockworkProvider, PAYER_PUBKEY} from "@clockwork-xyz/sdk";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const clockworkProvider = new ClockworkProvider(provider.wallet, provider.connection);
// 👇 will get fixed in future version of ClockworkProvider
clockworkProvider.threadProgram.provider.connection.opts = AnchorProvider.defaultOptions();
const program = anchor.workspace.Payments as Program<Payments>;


describe("payment", () => {
    it("It disburses payment from a Thread!", async () => {
        const [authority, authorityAta, mint, bob, bobAta] = await prepareMints();

        const [payment] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("payment"), // 👈 make sure it matches on the prog side
                authority.publicKey.toBuffer(),
                mint.toBuffer(),
                bob.toBuffer(),
            ],
            program.programId
        );

        console.log("program logs: solana logs -u devnet | grep " + program.programId.toString() + "\n\n");
        print_address("mint", mint);
        print_address("authorityAta", authorityAta);
        print_address("bob's token account", bobAta);

        try {
            const amount = BigInt(10_000);
            // Create a Payment
            await createPayment(
                payment,
                amount,
                mint,
                authority,
                authorityAta,
                bob,
            )

            // Create Disburse Thread: Ask Thread to disburse payment every 10s at ${amount} tokens.
            await createDisbursePaymentThread(
                authority,
                authorityAta,
                payment,
                mint,
                bob,
                bobAta
            );

            // Verifying that bob has received the tokens
            console.log(`Verifying that Thread distributed ${amount} tokens to Bob...`);
            await sleep(12);
            const bobAmount = await verifyAmount(bobAta, amount);
            console.log(`Bob has received ${bobAmount} tokens`);

            const newAmount = amount + BigInt(5_000);
            console.log(`Increasing payment from ${amount} to ${newAmount}`);
            await updatePayment(payment, newAmount);
            const paymentAmount = (await program.account.payment.fetch(payment)).amount;
            assert.equal(paymentAmount.toString(), newAmount.toString());

            // Verifying that bob has received the tokens
            console.log(`Verifying that Thread distributed ${newAmount} tokens to Bob...`);
            await sleep(12);
            const expectedAmount = (bobAmount + newAmount)
            const bobAmount2 = await verifyAmount(bobAta, expectedAmount);
            console.log(`Bob has received ${bobAmount2} tokens`);
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

const prepareMints = async (): Promise<[Signer, PublicKey, PublicKey, PublicKey, PublicKey]> => {
    const authority = provider.wallet.payer;

    const mint = await createMint(
        provider.connection,
        authority,
        authority.publicKey,
        null,
        9
    );

    const authorityAta = await createAssociatedTokenAccount(
        provider.connection,
        authority,
        mint,
        authority.publicKey,
    );

    const recipient = Keypair.generate().publicKey;
    const recipientAta = await getAssociatedTokenAddress(mint, recipient);

    // Mint to Authority
    await mintToChecked(
        provider.connection,
        authority,
        mint,
        authorityAta,
        authority.publicKey,
        1e9,
        9
    );

    return [authority, authorityAta, mint, recipient, recipientAta]
}

const createPayment = async (
    payment: PublicKey,
    amount: bigint,
    mint: PublicKey,
    authority: Signer,
    authorityAta: PublicKey,
    recipient: PublicKey,
) => {
    await program.methods
        .createPayment(new anchor.BN(amount.toString()))
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            clockwork: clockworkProvider.threadProgram.programId,
            payment: payment,
            mint: mint,
            authority: authority.publicKey,
            authorityTokenAccount: authorityAta,
            recipient: recipient
        })
        .rpc();
}

const createDisbursePaymentThread = async (
    authority: Signer,
    authorityAta: PublicKey,
    payment: PublicKey,
    mint: PublicKey,
    recipient: PublicKey,
    recipientAta: PublicKey,
) => {
    // const threadName = "payment";
    // For debug: use a fix thread name such as the above, when your code works!
    const date = new Date();
    const threadName = "payment_" + date.toLocaleDateString() + "-" + date.getHours() + ":" + date.getMinutes();

    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    const threadAuthority = authority.publicKey;
    const [threadAddress] = clockworkProvider.getThreadPDA(threadAuthority, threadName);

    // https://docs.rs/clockwork-utils/latest/clockwork_utils/static.PAYER_PUBKEY.html
    const payer = PAYER_PUBKEY;

    const targetIx = await program.methods
        .disbursePayment()
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            clockwork: clockworkProvider.threadProgram.programId,
            payer: payer,
            authority: threadAuthority,
            mint: mint,
            authorityTokenAccount: authorityAta,
            payment: payment,
            recipient: recipient,
            recipientTokenAccount: recipientAta,
            thread: threadAddress,
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
        threadName,
        [targetIx],
        trigger,
        threadSOLBudget
    );

    const threadAccount = await clockworkProvider.getThreadAccount(threadAddress);
    console.log("Thread: ", threadAccount);
    print_address("🤖 Program", program.programId.toString());
    print_thread_address("🧵 Thread", threadAddress);
}

const updatePayment = async (
    payment: PublicKey,
    amount: bigint,
) => {
    await program.methods
        .updatePayment(new anchor.BN(amount.toString()))
        .accounts({
            payment: payment
        })
        .rpc();
}

const verifyAmount = async (ata, expectedAmount) => {
    const amount = (await getAccount(provider.connection, ata)).amount;
    assert.equal(
        amount.toString(),
        expectedAmount.toString()
    );
    return amount;
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
