import { Keypair, PublicKey } from "@solana/web3.js";
import { ClockworkProvider } from "@clockwork-xyz/sdk";

const keypairFromFile = (path: string): Keypair => {
  return Keypair.fromSecretKey(
    Buffer.from(JSON.parse(require("fs").readFileSync(path, "utf-8")))
  );
};

// helpers
let lastThreadExec = BigInt(0);
const waitForThreadExec = async (
  clockworkProvider: ClockworkProvider,
  thread: PublicKey,
  maxWait: number = 60
) => {
  let i = 1;
  while (true) {
    const execContext = (await clockworkProvider.getThreadAccount(thread))
      .execContext;
    if (execContext) {
      if (
        lastThreadExec.toString() == "0" ||
        execContext.lastExecAt > lastThreadExec
      ) {
        lastThreadExec = execContext.lastExecAt;
        break;
      }
    }
    if (i == maxWait) throw Error("Timeout");
    i += 1;
    await new Promise((r) => setTimeout(r, i * 1000));
  }
};

export { keypairFromFile, waitForThreadExec };
