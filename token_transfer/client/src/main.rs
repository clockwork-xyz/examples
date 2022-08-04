use {
    anchor_lang::{prelude::Pubkey, solana_program::sysvar, InstructionData},
    anchor_spl::{associated_token, token},
    solana_client_helpers::{Client, ClientResult, RpcClient, SplToken},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    },
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("https://api.devnet.solana.com");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let recipient = Keypair::new().pubkey();
    let authority = token_transfer::state::Authority::pubkey();
    let queue =
        clockwork_scheduler::state::Queue::pubkey(authority, "token_transfer_queue".to_string());
    let task = clockwork_scheduler::state::Task::pubkey(queue, 0);
    let escrow = token_transfer::state::Escrow::pda(client.payer_pubkey(), recipient).0;

    // create token mint
    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    // Create ATAs
    let sender_token_account =
        client.create_associated_token_account(&client.payer(), &client.payer_pubkey(), &mint)?;
    let recipient_token_account =
        client.create_associated_token_account(&client.payer(), &recipient, &mint)?;

    // get vault associated token address
    let vault = anchor_spl::associated_token::get_associated_token_address(&escrow, &mint);

    create_queue(&client, authority, queue)?;

    create_escrow(&client, authority, escrow, mint, recipient, queue)?;

    deposit_funds(
        &client,
        escrow,
        mint,
        recipient,
        sender_token_account,
        vault,
    )?;

    create_task(
        &client,
        authority,
        escrow,
        queue,
        task,
        recipient,
        recipient_token_account,
        vault,
    )?;

    Ok(())
}

fn create_queue(client: &Client, authority: Pubkey, queue: Pubkey) -> ClientResult<()> {
    // create ix
    let ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: token_transfer::instruction::CreateQueue {}.data(),
    };

    send_and_confirm_tx(client, ix, "create_queue".to_string())?;

    Ok(())
}

fn create_escrow(
    client: &Client,
    authority: Pubkey,
    escrow: Pubkey,
    mint: Pubkey,
    recipient: Pubkey,
    queue: Pubkey,
) -> ClientResult<()> {
    // create ix
    let ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(queue, false),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: token_transfer::instruction::CreateEscrow {
            amount: LAMPORTS_PER_SOL,
            transfer_rate: 10000,
        }
        .data(),
    };

    send_and_confirm_tx(client, ix, "create_escrow".to_string())?;

    Ok(())
}

fn deposit_funds(
    client: &Client,
    escrow: Pubkey,
    mint: Pubkey,
    recipient: Pubkey,
    sender_token_account: Pubkey,
    vault: Pubkey,
) -> ClientResult<()> {
    // mint to sender's associated token account
    client.mint_to(
        &client.payer(),
        &mint,
        &sender_token_account,
        LAMPORTS_PER_SOL,
        9,
    )?;

    // create ix
    let ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(sender_token_account, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new(vault, false),
        ],
        data: token_transfer::instruction::DepositFunds {}.data(),
    };

    send_and_confirm_tx(client, ix, "deposit_funds".to_string())?;

    Ok(())
}

fn create_task(
    client: &Client,
    authority: Pubkey,
    escrow: Pubkey,
    queue: Pubkey,
    task: Pubkey,
    recipient: Pubkey,
    recipient_token_account: Pubkey,
    vault: Pubkey,
) -> ClientResult<()> {
    let ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(authority, false),
            AccountMeta::new_readonly(escrow, false),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(recipient, false),
            AccountMeta::new_readonly(recipient_token_account, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(task, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(vault, false),
        ],
        data: token_transfer::instruction::CreateTask {}.data(),
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
