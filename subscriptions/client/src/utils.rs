use {
    anchor_lang::prelude::Pubkey,
    clockwork_sdk::client::{Client, ClientResult},
    serde_json::Value,
    solana_sdk::signature::Keypair,
    solana_sdk::{instruction::Instruction, transaction::Transaction},
};

pub fn send_and_confirm_tx(
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
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=devnet",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
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

pub fn get_env_vars() -> (
    Client,
    Option<Pubkey>,
    Option<Pubkey>,
    Option<Pubkey>,
    Option<Pubkey>,
    Option<Pubkey>,
    Option<Pubkey>,
    Option<u64>,
) {
    let client = get_client();
    let subscription = get_address_env("SUBSCRIPTION".to_string());
    let subscription_thread = get_address_env("SUBSCRIPTION_THREAD".to_string());
    let subscription_bank = get_address_env("SUBSCRIPTION_BANK".to_string());
    let subscriber = get_address_env("SUBSCRIBER".to_string());
    let subscriber_token_account = get_address_env("SUBSCRIBER_TOKEN_ACCOUNT".to_string());
    let mint = get_address_env("MINT".to_string());
    let subscriber_id = get_id();
    return (
        client,
        subscription,
        subscription_thread,
        subscription_bank,
        subscriber,
        subscriber_token_account,
        mint,
        subscriber_id,
    );
}

pub fn get_id() -> Option<u64> {
    let id = std::env::var("SUBSCRIPTION_ID").unwrap();
    if id.is_empty() {
        return None;
    }
    let id = id.parse::<u64>().unwrap();
    return Some(id);
}

pub fn get_address_env(address: String) -> Option<Pubkey> {
    let arg = std::env::var(address).unwrap();
    if arg.is_empty() {
        return None;
    }
    let decoded = &bs58::decode(arg).into_vec().unwrap();
    return Some(Pubkey::new(decoded));
}

pub fn get_client() -> Client {
    let client_private_str = &*std::env::var("CLIENT_PRIVATE").unwrap();
    let client_private = &serde_json::from_str(client_private_str).unwrap();
    let mut key = vec![];
    if let Value::Array(arr) = client_private {
        for val in arr {
            if let Value::Number(value) = val {
                let a = value.as_u64().unwrap() as u8;
                key.push(a);
            }
        }
    }

    let keypair = Keypair::from_bytes(&key).unwrap();
    let client = Client::new(keypair, "https://api.devnet.solana.com".into());
    return client;
}

pub fn print_config(
    subscription: Pubkey,
    subscription_thread: Pubkey,
    subscription_bank: Pubkey,
    subscriber: Pubkey,
    subscriber_token_account: Pubkey,
    mint: Pubkey,
    subscription_id: u64,
) {
    println!("- - - - - - - - - - UPDATE YOUR .ENV FILE - - - - - - - - - -");
    println!("SUBSCRIPTION=\"{:?}\"", subscription);
    println!("SUBSCRIPTION_THREAD=\"{:?}\"", subscription_thread);
    println!("SUBSCRIPTION_BANK=\"{:?}\"", subscription_bank);
    println!("SUBSCRIBER=\"{:?}\"", subscriber);
    println!(
        "SUBSCRIBER_TOKEN_ACCOUNT=\"{:?}\"",
        subscriber_token_account
    );
    println!("MINT=\"{:?}\"", mint);
    println!("SUBSCRIPTION_ID=\"{}\"", subscription_id);
    println!("- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -");
}
