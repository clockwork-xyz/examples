import { expect } from "chai";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  getAssociatedTokenAddress,
  createTransferInstruction,
  createAssociatedTokenAccountIdempotentInstruction,
  mintTo,
} from "@solana/spl-token";
import { AnchorProvider } from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";
import { keypairFromFile, waitForThreadExec } from "./utils";


describe("spl-transfer", async () => {
  it("It transfers tokens every 10s", async () => {
    const connection = new Connection("http://localhost:8899", "processed");
    const payer = keypairFromFile(
      require("os").homedir() + "/.config/solana/id.json"
    );

    // Prepare clockworkProvider
    const provider = new AnchorProvider(
      connection,
      new NodeWallet(payer),
      AnchorProvider.defaultOptions()
    );
    const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);

    // Prepare source and dest
    const threadId = "spljs_" + new Date().getTime();
    const [thread] = clockworkProvider.getThreadPDA(
      provider.wallet.publicKey,
      threadId
    );
    console.log(`Thread id: ${threadId}, address: ${thread}`);

    // We will use the thread pda as the source and fund it with some tokens
    const source = thread;
    const [mint, sourceAta] = await fundSource(connection, payer, source);
    console.log(`source: ${source}, sourceAta: ${sourceAta}`);

    // Prepare dest
    const dest = Keypair.generate().publicKey;
    const destAta = await getAssociatedTokenAddress(mint, dest);
    const targetIx0 = createAssociatedTokenAccountIdempotentInstruction(
      PAYER_PUBKEY,
      destAta,
      dest,
      mint,
    );
    console.log(`dest: ${dest}, destAta: ${destAta}`)

    // 1️⃣  Create a transfer instruction.
    const amount = 1e8;
    const targetIx = createTransferInstruction(sourceAta, destAta, source, amount);

    // 2️⃣  Define a trigger condition for the thread.
    const trigger = {
      cron: {
        schedule: "*/10 * * * * * *",
        skippable: true,
      },
    };

    // 3️⃣  Create the thread.
    try {
      const ix = await clockworkProvider.threadCreate(
        provider.wallet.publicKey, // authority
        threadId, // id
        [targetIx0, targetIx], // instructions to execute
        trigger, // trigger condition
        LAMPORTS_PER_SOL // amount to fund the thread with for execution fees
      );
      const tx = new Transaction().add(ix);
      const sig = await clockworkProvider.anchorProvider.sendAndConfirm(tx);
      console.log(`Thread created: ${sig}`);
    } catch (e) {
      // ❌
      // 'Program log: Instruction: ThreadCreate',
      // 'Program 11111111111111111111111111111111 invoke [2]',
      // 'Allocate: account Address { address: ..., base: None } already in use'
      //
      // -> If you encounter this error, the address you are trying to assign to the newly created thread is already in use.
      //    You can change the threadId, to generate a new account address.
      // -> OR update the thread with a ThreadUpdate instruction (more on this in future guide)
      console.error(e);
    }

    // Check balance for the next 3 executions
    let checkAmount = 1e8;
    for (let i = 1; i < 4; i++) {
      await waitForThreadExec(clockworkProvider, thread);
      const destAtaAcc = await getAccount(connection, destAta);
      expect(destAtaAcc.amount.toString()).to.eq(checkAmount.toString());
      console.log(`destAta balance: ${destAtaAcc.amount}`);
      checkAmount += amount;
    }
  });
});


const fundSource = async (
  connection: Connection,
  payer: Keypair,
  source: PublicKey
): Promise<[PublicKey, PublicKey]> => {
  const mint = await createMint(
    connection,
    payer, // (Signer)
    payer.publicKey, // mint authority
    null, // freeze authority
    8 // decimals
  );
  const sourceAta = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    source,
    true // we set this to true as our source is the thread PDA
  );
  await mintTo(connection, payer, mint, sourceAta.address, payer, 100 * 1e8);
  return [mint, sourceAta.address];
};
