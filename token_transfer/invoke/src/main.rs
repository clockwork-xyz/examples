use std::vec;

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
    let amount = LAMPORTS_PER_SOL;
    let transfer_rate = 1000000;

    // Create Client
    let client = RpcClient::new("http://localhost:8899");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), amount)?;

    // Derive PDAs
    let recipient = Keypair::new();
    let recipient_pubkey = recipient.pubkey();
    let authority_pubkey = token_transfer::state::Authority::pda().0;
    let escrow_pubkey =
        token_transfer::state::Escrow::pda(client.payer_pubkey(), recipient_pubkey).0;
    let manager_pubkey = cronos_scheduler::state::Manager::pda(authority_pubkey).0;

    // create token mint
    let mint = client.create_token_mint(&client.payer_pubkey(), 9)?;

    // Create ATAs
    let sender_token_account_pubkey = client.create_associated_token_account(
        &client.payer(),
        &client.payer_pubkey(),
        &mint.pubkey(),
    )?;
    let recipient_token_account_pubkey = client.create_associated_token_account(
        &client.payer(),
        &recipient_pubkey,
        &mint.pubkey(),
    )?;

    // get vault associated token address
    let vault_pubkey =
        anchor_spl::associated_token::get_associated_token_address(&escrow_pubkey, &mint.pubkey());

    initialize(&client, authority_pubkey, manager_pubkey)?;

    deposit(
        &client,
        amount,
        authority_pubkey,
        escrow_pubkey,
        manager_pubkey,
        mint.pubkey(),
        recipient_pubkey,
        sender_token_account_pubkey,
        transfer_rate,
        vault_pubkey,
    )?;

    auto_withdraw(
        &client,
        authority_pubkey,
        escrow_pubkey,
        manager_pubkey,
        recipient_pubkey,
        recipient_token_account_pubkey,
        vault_pubkey,
    )?;

    Ok(())
}

fn initialize(
    client: &Client,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
) -> ClientResult<()> {
    // create ix for initialize ix
    let initialize_ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new(authority_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new(manager_pubkey, false),
        ],
        data: token_transfer::instruction::Initialize {}.data(),
    };

    // Create tx for initialize ix
    let mut tx = Transaction::new_with_payer(&[initialize_ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm initialize tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "Initialize ix: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            sig
        ),
        Err(err) => println!("Initialize ix: ❌ {:#?}", err),
    }

    Ok(())
}

fn deposit(
    client: &Client,
    amount: u64,
    authority_pubkey: Pubkey,
    escrow_pubkey: Pubkey,
    manager_pubkey: Pubkey,
    mint_pubkey: Pubkey,
    recipient_pubkey: Pubkey,
    sender_token_account_pubkey: Pubkey,
    transfer_rate: u64,
    vault_pubkey: Pubkey,
) -> ClientResult<()> {
    // Derive PDAs
    let disburse_queue_pubkey = cronos_scheduler::state::Queue::pda(manager_pubkey, 0).0;

    // mint to sender's associated token account
    client.mint_to(
        &client.payer(),
        &mint_pubkey,
        &sender_token_account_pubkey,
        amount,
        9,
    )?;

    // create deposit ix
    let create_payment_ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new(escrow_pubkey, false),
            AccountMeta::new(manager_pubkey, false),
            AccountMeta::new_readonly(mint_pubkey, false),
            AccountMeta::new_readonly(recipient_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(sender_token_account_pubkey, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new(vault_pubkey, false),
            // Extra Accounts
            AccountMeta::new(disburse_queue_pubkey, false),
        ],
        data: token_transfer::instruction::Deposit {
            amount,
            transfer_rate,
        }
        .data(),
    };

    println!(
        "vault: https://explorer.solana.com/address/{}?cluster=custom",
        vault_pubkey
    );

    // Create deposit tx
    let mut tx = Transaction::new_with_payer(&[create_payment_ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm deposit tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "deposit ix: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            sig
        ),
        Err(err) => println!("deposit ix: ❌ {:#?}", err),
    }

    Ok(())
}

fn auto_withdraw(
    client: &Client,
    authority_pubkey: Pubkey,
    escrow_pubkey: Pubkey,
    manager_pubkey: Pubkey,
    recipient_pubkey: Pubkey,
    recipient_token_account_pubkey: Pubkey,
    vault_pubkey: Pubkey,
) -> ClientResult<()> {
    // Derive PDAs
    let disburse_queue_pubkey = cronos_scheduler::state::Queue::pda(manager_pubkey, 0).0;
    let disburse_fee_pubkey = cronos_scheduler::state::Fee::pda(disburse_queue_pubkey).0;
    let disburse_task_pubkey = cronos_scheduler::state::Task::pda(disburse_queue_pubkey, 0).0;

    let auto_withdraw_ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(authority_pubkey, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(escrow_pubkey, false),
            AccountMeta::new(manager_pubkey, false),
            AccountMeta::new_readonly(recipient_pubkey, false),
            AccountMeta::new_readonly(recipient_token_account_pubkey, false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(vault_pubkey, false),
            // Extra accounts
            AccountMeta::new(disburse_fee_pubkey, false),
            AccountMeta::new(disburse_queue_pubkey, false),
            AccountMeta::new(disburse_task_pubkey, false),
        ],
        data: token_transfer::instruction::AutoWithdraw {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[auto_withdraw_ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm deposit tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "auto_withdraw ix: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            sig
        ),
        Err(err) => println!("auto_withdraw ix: ❌ {:#?}", err),
    }

    Ok(())
}
