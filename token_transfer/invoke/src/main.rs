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

    // Initialize Accounts
    let recipient = Keypair::new();
    let recipient_pubkey = recipient.pubkey();
    let authority_pubkey = token_transfer::state::Authority::pda().0;
    let manager_pubkey = cronos_scheduler::state::Manager::pda(authority_pubkey).0;

    initialize(&client, recipient_pubkey, authority_pubkey, manager_pubkey)?;

    create_payment(
        &client,
        recipient_pubkey,
        authority_pubkey,
        manager_pubkey,
        amount,
        transfer_rate,
    )?;

    Ok(())
}

fn initialize(
    client: &Client,
    recipient_pubkey: Pubkey,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
) -> ClientResult<()> {
    // create ix for initialize ix
    let initialize_ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new(authority_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(recipient_pubkey, false),
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
            "Initialize ix: ✅ https://explorer.solana.com/tx/{}?cluster=devnet",
            sig
        ),
        Err(err) => println!("Initialize ix: ❌ {:#?}", err),
    }

    Ok(())
}

fn create_payment(
    client: &Client,
    recipient_pubkey: Pubkey,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
    amount: u64,
    transfer_rate: u64,
) -> ClientResult<()> {
    // derive PDAs
    let escrow_pubkey = token_transfer::state::Escrow::pda().0;
    let create_payment_queue_pubkey = cronos_scheduler::state::Queue::pda(manager_pubkey, 0).0;
    let create_payment_fee_pubkey =
        cronos_scheduler::state::Fee::pda(create_payment_queue_pubkey).0;
    let create_payment_task_pubkey =
        cronos_scheduler::state::Task::pda(create_payment_queue_pubkey, 0).0;

    // create mint
    let mint = client.create_token_mint(&client.payer_pubkey(), 9)?;

    // create token accounts
    let sender_token_account =
        client.create_token_account(&client.payer_pubkey(), &mint.pubkey())?;
    let recipient_token_account = client.create_token_account(&recipient_pubkey, &mint.pubkey())?;

    let vault_pubkey =
        anchor_spl::associated_token::get_associated_token_address(&escrow_pubkey, &mint.pubkey());

    println!(
        "recipient associated token account: https://explorer.solana.com/address/{}?cluster=devnet",
        recipient_token_account.pubkey()
    );
    println!(
        "vault: https://explorer.solana.com/address/{}?cluster=devnet",
        vault_pubkey
    );

    // mint to sender's token account
    client.mint_to(
        &client.payer(),
        &mint.pubkey(),
        &sender_token_account.pubkey(),
        amount,
        9,
    )?;

    // create ix for create_payment_ix ix
    let create_payment_ix = Instruction {
        program_id: token_transfer::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new(escrow_pubkey, false),
            AccountMeta::new(manager_pubkey, false),
            AccountMeta::new_readonly(mint.pubkey(), false),
            AccountMeta::new_readonly(recipient_pubkey, false),
            AccountMeta::new_readonly(recipient_token_account.pubkey(), false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(sender_token_account.pubkey(), false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new(vault_pubkey, false),
            // Extra accounts
            AccountMeta::new(create_payment_fee_pubkey, false),
            AccountMeta::new(create_payment_queue_pubkey, false),
            AccountMeta::new(create_payment_task_pubkey, false),
        ],
        data: token_transfer::instruction::CreatePayment {
            amount,
            transfer_rate,
        }
        .data(),
    };

    // Create create_payment tx
    let mut tx = Transaction::new_with_payer(&[create_payment_ix], Some(&client.payer_pubkey()));
    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm create_payment tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "create_payment ix: ✅ https://explorer.solana.com/tx/{}?cluster=devnet",
            sig
        ),
        Err(err) => println!("create_payment ix: ❌ {:#?}", err),
    }

    Ok(())
}
