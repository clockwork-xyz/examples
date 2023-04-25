import { PublicKey } from "@solana/web3.js";
declare const print_address: (label: any, address: any) => void;
declare const print_thread: (clockworkProvider: any, address: any) => Promise<void>;
declare const print_tx: (label: any, address: any) => void;
declare const stream_program_logs: (programId: any) => void;
declare const verifyAmount: (connection: any, ata: any, expectedAmount: any) => Promise<bigint>;
declare const waitForThreadExec: (clockworkProvider: any, thread: PublicKey, maxWait?: number) => Promise<void>;
export { print_address, print_thread, print_tx, stream_program_logs, verifyAmount, waitForThreadExec, };
