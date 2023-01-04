use {
    anchor_lang::{prelude::*, solana_program::sysvar},
    clockwork_client::{Client, ClientResult},
    solana_sdk::{
        instruction::Instruction, signature::Keypair, signer::Signer, transaction::Transaction,
    },
};

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

pub fn print_market_keys(market_keys: &MarketKeys) -> ClientResult<()> {
    println!("serum explorer: https://serum-explorer.vercel.app/market/{}?network=custom&customRPC=http%3A%2F%2Flocalhost%3A8899", market_keys.market);
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "market".to_string(),
        market_keys.market
    );
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "event_thread".to_string(),
        market_keys.event_q
    );
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "mint_a_vault".to_string(),
        market_keys.coin_vault
    );
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "mint_a_wallet".to_string(),
        market_keys.pc_wallet_key.pubkey()
    );
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "mint_b_vault".to_string(),
        market_keys.pc_vault
    );
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        "mint_b_wallet".to_string(),
        market_keys.coin_wallet_key.pubkey()
    );
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

pub fn gen_listing_params(
    client: &Client,
    program_id: &Pubkey,
    payer: &Pubkey,
    _coin_mint: &Pubkey,
    _pc_mint: &Pubkey,
) -> ClientResult<(ListingKeys, Vec<Instruction>)> {
    let (market_key, create_market) = create_dex_account(client, program_id, payer, 376)?;
    let (req_q_key, create_req_q) = create_dex_account(client, program_id, payer, 640)?;
    let (event_q_key, create_event_q) = create_dex_account(client, program_id, payer, 1 << 20)?;
    let (bids_key, create_bids) = create_dex_account(client, program_id, payer, 1 << 16)?;
    let (asks_key, create_asks) = create_dex_account(client, program_id, payer, 1 << 16)?;
    let (vault_signer_nonce, vault_signer) = {
        let mut i = 0;
        loop {
            assert!(i < 100);
            if let Ok(pk) = anchor_spl::dex::serum_dex::state::gen_vault_signer_key(
                i,
                &market_key.pubkey(),
                program_id,
            ) {
                break (i, pk);
            }
            i += 1;
        }
    };
    let info = ListingKeys {
        market_key,
        req_q_key,
        event_q_key,
        bids_key,
        asks_key,
        vault_signer,
        vault_signer_nonce,
    };
    let instructions = vec![
        create_market,
        create_req_q,
        create_event_q,
        create_bids,
        create_asks,
    ];
    Ok((info, instructions))
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

pub fn init_open_orders_ix(
    program_id: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    market: &Pubkey,
) -> ClientResult<Instruction> {
    let data = anchor_spl::dex::serum_dex::instruction::MarketInstruction::InitOpenOrders.pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*open_orders, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(sysvar::rent::ID, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub struct ListingKeys {
    pub market_key: Keypair,
    pub req_q_key: Keypair,
    pub event_q_key: Keypair,
    pub bids_key: Keypair,
    pub asks_key: Keypair,
    pub vault_signer: Pubkey,
    pub vault_signer_nonce: u64,
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
    pub coin_wallet_key: Keypair,
    pub pc_mint: Pubkey,
    pub pc_vault: Pubkey,
    pub pc_wallet_key: Keypair,
    pub vault_signer: Pubkey,
}
