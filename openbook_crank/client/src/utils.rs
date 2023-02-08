use {
    anchor_lang::prelude::*,
    clockwork_client::{Client, ClientResult},
    solana_sdk::{
        instruction::Instruction,
        signature::{read_keypair_file, Keypair},
        transaction::Transaction,
    },
};

pub fn openbook_dex_pk() -> Pubkey {
    anchor_spl::dex::ID
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
        Ok(sig) => println!("{} tx: ✅ https://explorer.solana.com/tx/{}", label, sig),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
}

pub fn print_explorer_link(address: Pubkey, label: String) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}",
        label.to_string(),
        address,
    );

    Ok(())
}

pub fn default_client() -> Client {
    #[cfg(not(feature = "localnet"))]
    let host = "https://api.mainnet-beta.solana.com";
    #[cfg(feature = "localnet")]
    let host = "http://localhost:8899";

    let config_file = solana_cli_config::CONFIG_FILE.as_ref().unwrap().as_str();
    let config = solana_cli_config::Config::load(config_file).unwrap();
    let payer = read_keypair_file(&config.keypair_path).unwrap();
    Client::new(payer, host.into())
}

#[derive(Debug)]
pub struct MarketKeys {
    pub market: Pubkey,
    // pub req_q: Pubkey,
    pub event_q: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub coin_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub coin_wallet: Pubkey,
    pub pc_mint: Pubkey,
    pub pc_vault: Pubkey,
    pub pc_wallet: Pubkey,
    pub vault_signer: Pubkey,
}
