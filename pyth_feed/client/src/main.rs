use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_sdk::{Client, ClientResult},
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

    // create queue that listens for account changes for a pyth pricing feed
    create_feed(&client)?;

    Ok(())
}

fn create_feed(client: &Client) -> ClientResult<()> {
    let feed_pubkey = pyth_feed::state::Feed::pubkey();
    let create_feed_ix = Instruction {
        program_id: pyth_feed::ID,
        accounts: vec![
            AccountMeta::new_readonly(clockwork_sdk::queue_program::ID, false),
            AccountMeta::new(feed_pubkey, false),
            AccountMeta::new(
                clockwork_sdk::queue_program::accounts::Queue::pubkey(feed_pubkey, "feed".into()),
                false,
            ),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: pyth_feed::instruction::CreateFeed {
            // SOL/USD price feed
            pyth_feed: Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap(),
        }
        .data(),
    };
    sign_send_and_confirm_tx(
        &client,
        [create_feed_ix].to_vec(),
        None,
        "create_feed".into(),
    )?;
    Ok(())
}

pub fn print_explorer_link(address: Pubkey, label: String) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
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
