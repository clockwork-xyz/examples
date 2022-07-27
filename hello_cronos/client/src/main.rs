use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            system_program, sysvar,
        },
        InstructionData,
    },
    solana_client_helpers::{Client, ClientResult, RpcClient},
    solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, transaction::Transaction},
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("https://api.devnet.solana.com");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), LAMPORTS_PER_SOL)?;

    // Initialize Accounts
    let authority = hello_cronos::state::Authority::pda().0;
    let manager = cronos_scheduler::state::Manager::pda(authority).0;
    let message_queue = cronos_scheduler::state::Queue::pda(manager, 0).0;
    let message_fee = cronos_scheduler::state::Fee::pda(message_queue).0;
    let message_task = cronos_scheduler::state::Task::pda(message_queue, 0).0;

    // Create ix
    let ix = Instruction {
        program_id: hello_cronos::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(authority, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(message_fee, false),
            AccountMeta::new(message_queue, false),
            AccountMeta::new(manager, false),
            AccountMeta::new(message_task, false),
        ],
        data: hello_cronos::instruction::Initialize {}.data(),
    };

    // Create tx
    let mut tx = Transaction::new_with_payer(&[ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("✅ https://explorer.solana.com/tx/{}?cluster=devnet", sig),
        Err(err) => println!("❌ {:#?}", err),
    }

    Ok(())
}
