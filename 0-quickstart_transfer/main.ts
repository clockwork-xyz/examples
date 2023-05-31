import { expect } from "chai";
import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    SystemProgram,
    Transaction,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { ClockworkProvider, PAYER_PUBKEY } from "@clockwork-xyz/sdk";

const connection = new Connection("http://localhost:8899", "processed");
const payer = Keypair.fromSecretKey(
    Buffer.from(JSON.parse(require("fs").readFileSync(
        require("os").homedir() + "/.config/solana/id.json",
        "utf-8"
    )))
);

// Prepare clockworkProvider
const provider = new anchor.AnchorProvider(
    connection,
    new NodeWallet(payer),
    anchor.AnchorProvider.defaultOptions()
);
const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);


describe("transfer", async () => {
    it("Transfers SOL every 10 seconds", async () => {
      const threadId = "sol_transferjs" + new Date().getTime();
      const [threadAddress] = clockworkProvider.getThreadPDA(
          payer.publicKey,   // authority
          threadId
       )
  
      const recipient = Keypair.generate().publicKey;
      console.log(`🫴  recipient: ${recipient.toString()}\n`);

      // 1️⃣  Prepare an instruction to be automated.
      const transferIx = SystemProgram.transfer({
          fromPubkey: PAYER_PUBKEY,
          toPubkey: recipient,
          lamports: LAMPORTS_PER_SOL,
      });
  
      // 2️⃣  Define a trigger condition.
      const trigger = {
          cron: {
              schedule: "*/10 * * * * * *",
              skippable: true,
          },
      };
  
      // 3️⃣ Create the thread.
      const ix = await clockworkProvider.threadCreate(
          payer.publicKey,           // authority
          threadId,                  // id
          [transferIx],              // instructions
          trigger,                   // trigger
          50 * LAMPORTS_PER_SOL,      // amount to fund the thread with
      );
      const tx = new Transaction().add(ix);
      const signature = await clockworkProvider.anchorProvider.sendAndConfirm(tx);
      console.log(`🗺️  explorer: https://app.clockwork.xyz/threads/${threadAddress}?cluster=custom&customUrl=${connection.rpcEndpoint}\n`);
      
      // Check balance of recipient address
      await new Promise((resolve) => setTimeout(resolve, 10 * 1000));
      let balance = await connection.getBalance(recipient) / LAMPORTS_PER_SOL;
      console.log(`✅ recipient balance: ${balance} SOL\n`);
      expect(balance).to.eq(1);

      await new Promise((resolve) => setTimeout(resolve, 10 * 1000));
      balance = await connection.getBalance(recipient) / LAMPORTS_PER_SOL;
      console.log(`✅ recipient balance: ${balance} SOL\n`);
      expect(balance).to.eq(2);
    });
  });
