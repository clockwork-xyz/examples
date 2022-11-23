use clockwork_sdk::thread_program::accounts::Trigger;

use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        transaction::Transaction,
    },
    std::str::FromStr,
};

fn main() -> ClientResult<()> {
    // Create Client
    let payer = Keypair::new();
    let client = Client::new(payer, "https://api.devnet.solana.com".into());
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    create_feed(&client)?;

    Ok(())
}

fn create_feed(client: &Client) -> ClientResult<()> {
    // SOL/USD price feed
    let sol_usd_pubkey = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();

    let stat_pubkey = stats::objects::Stat::pubkey(sol_usd_pubkey, client.payer_pubkey());
    let stat_thread_pubkey = clockwork_sdk::thread_program::accounts::Thread::pubkey(
        client.payer_pubkey(),
        "stats".into(),
    );

    print_explorer_link(stat_thread_pubkey, "stat_thread".into())?;

    // airdrop thread
    client.airdrop(&stat_thread_pubkey, 2 * LAMPORTS_PER_SOL)?;

    let initialize_ix = Instruction {
        program_id: stats::ID,
        accounts: vec![
            AccountMeta::new_readonly(sol_usd_pubkey, false),
            AccountMeta::new(stat_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: stats::instruction::Initialize {
            lookback_window: 3600 as i64,
        }
        .data(),
    };

    let create_thread_ix = clockwork_sdk::client::thread_program::instruction::thread_create(
        client.payer_pubkey(),
        "stats".into(),
        Instruction {
            program_id: stats::ID,
            accounts: vec![
                AccountMeta::new(stat_pubkey, false),
                AccountMeta::new(sol_usd_pubkey, false),
                AccountMeta::new(stat_thread_pubkey, true),
                AccountMeta::new_readonly(sol_usd_pubkey, false),
            ],
            data: stats::instruction::Calc {}.data(),
        }
        .into(),
        client.payer_pubkey(),
        stat_thread_pubkey,
        Trigger::Account {
            address: sol_usd_pubkey,
            offset: 4 + 8 + 8 + 4 + 4 + 4 + 4,
            size: 8,
        },
    );

    sign_send_and_confirm_tx(
        &client,
        [initialize_ix, create_thread_ix].to_vec(),
        None,
        "init stat account and stat thread".into(),
    )?;

    Ok(())
}

pub fn print_explorer_link(address: Pubkey, label: String) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=devnet",
        label.to_string(),
        address
    );

    Ok(())
}

pub fn sign_send_and_confirm_tx(
    client: &Client,
    ix: Vec<Instruction>,
    signers: Option<Vec<&Keypair>>,
    label: String,
) -> ClientResult<()> {
    let mut tx;

    match signers {
        Some(signer_keypairs) => {
            tx = Transaction::new_signed_with_payer(
                &ix,
                Some(&client.payer_pubkey()),
                &signer_keypairs,
                client.get_latest_blockhash().unwrap(),
            );
        }
        None => {
            tx = Transaction::new_with_payer(&ix, Some(&client.payer_pubkey()));
        }
    }

    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
}
