use {
    anchor_lang::prelude::*,
    anchor_spl::dex::serum_dex::state::OpenOrders,
    clockwork_client::{Client, ClientResult},
    solana_sdk::{
        instruction::Instruction,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        transaction::Transaction,
    },
    std::mem::size_of,
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

pub fn create_dex_account(
    client: &Client,
    program_id: &Pubkey,
    payer: &Pubkey,
    unpadded_len: usize,
) -> ClientResult<(Keypair, Instruction)> {
    let len = unpadded_len + 12;
    let key = Keypair::new();
    let create_account_instr = solana_sdk::system_instruction::create_account(
        payer,
        &key.pubkey(),
        client.get_minimum_balance_for_rent_exemption(len)?,
        len as u64,
        program_id,
    );
    Ok((key, create_account_instr))
}

pub fn init_dex_account(client: &Client, orders: &mut Option<Pubkey>) -> ClientResult<()> {
    let orders_keypair;
    let mut ix = Vec::new();
    let mut signers = Vec::new();

    let orders_pk = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) = create_dex_account(
                client,
                &anchor_spl::dex::ID,
                &client.payer_pubkey(),
                size_of::<OpenOrders>(),
            )?;
            orders_keypair = orders_key;
            signers.push(&orders_keypair);
            ix.push(instruction);
            orders_keypair.pubkey()
        }
    };

    *orders = Some(orders_pk);

    signers.push(client.payer());

    sign_send_and_confirm_tx(client, ix, Some(signers), "init_dex_account".to_string())?;

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

pub fn print_explorer_link(address: Pubkey, label: String) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}",
        label.to_string(),
        address,
    );

    Ok(())
}
#[derive(Debug)]
pub struct MarketKeys {
    pub market: Pubkey,
    pub req_q: Pubkey,
    pub event_q: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub coin_mint: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_mint: Pubkey,
    pub pc_vault: Pubkey,
    pub vault_signer: Pubkey,
}
