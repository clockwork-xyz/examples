use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{
            instruction::{AccountMeta, Instruction},
            native_token::LAMPORTS_PER_SOL,
            system_program,
        },
        InstructionData,
    },
    solana_client_helpers::{Client, ClientResult, RpcClient},
    solana_sdk::{signature::Keypair, transaction::Transaction},
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("https://api.devnet.solana.com");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let authority = hello_clockwork::state::Authority::pubkey();
    let queue = clockwork_scheduler::state::Queue::pubkey(authority, "hello_queue".to_string());
    let task = clockwork_scheduler::state::Task::pubkey(queue, 0);

    create_queue(&client, authority, queue)?;

    create_task(&client, authority, task, queue)?;

    Ok(())
}

fn create_queue(client: &Client, authority: Pubkey, queue: Pubkey) -> ClientResult<()> {
    // Create ix
    let ix = Instruction {
        program_id: hello_clockwork::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: hello_clockwork::instruction::CreateQueue {}.data(),
    };

    send_and_confirm_tx(client, ix, "create_queue".to_string())?;

    Ok(())
}

fn create_task(
    client: &Client,
    authority: Pubkey,
    task: Pubkey,
    queue: Pubkey,
) -> ClientResult<()> {
    // Create ix
    let ix = Instruction {
        program_id: hello_clockwork::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(task, false),
        ],
        data: hello_clockwork::instruction::CreateTask {}.data(),
    };

    send_and_confirm_tx(client, ix, "create_task".to_string())?;

    Ok(())
}

fn send_and_confirm_tx(client: &Client, ix: Instruction, label: String) -> ClientResult<()> {
    // Create tx
    let mut tx = Transaction::new_with_payer(&[ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=devnet",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }

    Ok(())
}
