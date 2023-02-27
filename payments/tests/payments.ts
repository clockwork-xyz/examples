import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor./";
import {Payments} from "../target/types/payments";
import {
    Keypair,
    SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
    createMint,
    createAssociatedTokenAccount,
    getAssociatedTokenAddress,
    mintToChecked,
    TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

// 👇 The new import
import {getThreadAddress, CLOCKWORK_THREAD_PROGRAM_ID, createThread} from "@clockwork-xyz/sdk";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import {expect} from "chai";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Payments as Program<Payments>;


describe("payment", () => {
    it("It disburses payment!", async () => {
        const [mintPubkey, authorityTokenAccount, recipientPubkey, recipientAtaPubkey] = await prepareMints();
        const [paymentPubkey] = PublicKey.findProgramAddressSync(
            [
                anchor.utils.bytes.utf8.encode("payment"), // 👈 make sure it matches on the prog side
                provider.publicKey.toBuffer(),
                mintPubkey.toBuffer(),
                recipientPubkey.toBuffer(),
            ],
            program.programId
        );

        console.log("mintPubkey:" + mintPubkey.toString());
        console.log("authorityTokenAccount:" + authorityTokenAccount.toString());
        console.log("recipientPubkey:" + recipientPubkey.toString());
        console.log("recipientAtaPubkey:" + recipientAtaPubkey.toString());
        console.log("paymentPubkey:" + paymentPubkey.toString());

        // Create a Payment
        await createPayment(
            authorityTokenAccount,
            mintPubkey,
            paymentPubkey,
            recipientPubkey,
        )

        // Create Disburse Thread
        await createDisbursePaymentThread(
            authorityTokenAccount,
            mintPubkey,
            paymentPubkey,
            recipientPubkey,
            recipientAtaPubkey
        );

        // wait 10 seconds to update payment
        console.log("wait 10 seconds to update payment");
        await sleep(10);
        await updatePayment(paymentPubkey);
    });
});

const prepareMints = async (): Promise<[PublicKey, PublicKey, PublicKey, PublicKey]> => {
    // create token mint
    const mintPubkey = await createMint(
        provider.connection,
        provider.wallet.payer,
        provider.publicKey,
        null,
        9 // decimals
    );

    // Create authority token account
    const authorityTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        mintPubkey,
        provider.publicKey
    );

    // Get recipient's ATA
    const recipientPubkey = Keypair.generate().publicKey;
    const recipientAtaPubkey = await getAssociatedTokenAddress(
        mintPubkey, // mint
        provider.publicKey // owner
    );

    await mintToChecked(
        provider.connection, // connection
        provider.wallet.payer, // fee payer
        mintPubkey, // mint
        recipientAtaPubkey, // receiver (sholud be a token account)
        provider.publicKey, // mint authority
        1e9, // amount. if your decimals is 8, you mint 10^8 for 1 token.
        9 // decimals
    );

    return [mintPubkey, authorityTokenAccount, recipientPubkey, recipientAtaPubkey]
}

const createPayment = async (authorityTokenAccount: PublicKey,
                             mintPubkey: PublicKey,
                             paymentPubkey: PublicKey,
                             recipientPubkey: PublicKey,
) => {
    try {
        const createPaymentTx = await program.methods.createPayment(new anchor.BN(10_000))
            .accounts({
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                rent: SYSVAR_RENT_PUBKEY,
                clockwork: CLOCKWORK_THREAD_PROGRAM_ID,
                signer: provider.publicKey,
                authority: provider.publicKey,
                mint: mintPubkey,
                authorityTokenAccount: authorityTokenAccount,
                payment: paymentPubkey,
                recipient: recipientPubkey
            })
            .rpc();
    } catch (e) {
        console.error(e);
        expect.fail(e);
    }
}

const createDisbursePaymentThread = async (authorityTokenAccount: PublicKey,
                                           mintPubkey: PublicKey,
                                           paymentPubkey: PublicKey,
                                           recipientPubkey: PublicKey,
                                           recipientAtaPubkey: PublicKey,
) => {
    const threadLabel = "payment_22-2257";
    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    const threadAuthority = provider.publicKey;
    const threadAddress = getThreadAddress(threadAuthority, threadLabel);

    const targetIx = await program.methods.disbursePayment()
        .accounts({
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            clockwork: CLOCKWORK_THREAD_PROGRAM_ID,
            signer: provider.publicKey,
            authority: threadAuthority,
            mint: mintPubkey,
            authorityTokenAccount: authorityTokenAccount,
            payment: paymentPubkey,
            recipient: recipientPubkey,
            recipientTokenAccount: recipientAtaPubkey,
            thread: threadAddress,
        })
        .instruction();

    const trigger = {
        cron: {
            schedule: "*/10 * * * * * *",
            skippable: true,
        },
    }

    const createThreadIx = createThread({
        instruction: targetIx,
        trigger: trigger,
        threadName: threadLabel,
        threadAuthority: threadAuthority
    }, provider);
    try {
        const r = await createThreadIx;
        print_address("🤖 Program", program.programId.toString());
        print_thread_address("🧵 Thread", threadAddress);
        print_tx("✍️ Tx", r.transaction);
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

const updatePayment = async (paymentPubkey: PublicKey) => {
    try {
        const tx = await program.methods.updatePayment(new anchor.BN(50_000))
            .accounts({
                payment: paymentPubkey
            })
            .rpc();
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
