use {
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
        InstructionData,
    },
    solana_client_helpers::{Client, ClientResult, RpcClient},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        transaction::Transaction,
    },
};

fn main() -> ClientResult<()> {
    // Create Client
    #[cfg(feature = "devnet")]
    let client = RpcClient::new("https://api.devnet.solana.com");
    #[cfg(not(feature = "devnet"))]
    let client = RpcClient::new("http://localhost:8899");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Initialize the event_stream program
    initialize(&client)?;

    // Ping a new event every second.
    for _ in 0..5 {
        let one_sec = std::time::Duration::from_secs(1);
        std::thread::sleep(one_sec);
        ping(&client)?;
    }

    Ok(())
}

fn initialize(client: &Client) -> ClientResult<()> {
    let authority_pubkey = event_stream::state::Authority::pubkey();
    let initialize_ix = Instruction {
        program_id: event_stream::ID,
        accounts: vec![
            AccountMeta::new(authority_pubkey, false),
            AccountMeta::new_readonly(clockwork_sdk::id::ID, false),
            AccountMeta::new(event_stream::state::Event::pubkey(), false),
            AccountMeta::new(
                clockwork_sdk::state::Queue::pubkey(authority_pubkey, "events".into()),
                false,
            ),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: event_stream::instruction::Initialize {}.data(),
    };
    sign_send_and_confirm_tx(&client, [initialize_ix].to_vec(), None, "initialize".into())?;
    Ok(())
}

fn ping(client: &Client) -> ClientResult<()> {
    let ping_ix = Instruction {
        program_id: event_stream::ID,
        accounts: vec![
            AccountMeta::new(event_stream::state::Event::pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
        ],
        data: event_stream::instruction::Ping {}.data(),
    };
    sign_send_and_confirm_tx(&client, [ping_ix].to_vec(), None, "ping".into())?;
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

    // Send and confirm initialize tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
}
