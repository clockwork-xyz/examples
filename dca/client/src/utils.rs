use {
    anchor_lang::prelude::*,
    solana_client_helpers::{Client, ClientResult},
    solana_sdk::{
        instruction::Instruction, signature::Keypair, signer::Signer, transaction::Transaction,
    },
};

pub fn sign_send_and_confirm_tx(
    client: &Client,
    ix: Vec<Instruction>,
    signers: Option<Vec<&Keypair>>,
    tx_label: String,
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
            tx_label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", tx_label, err),
    }
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
    let (vault_signer_nonce, vault_signer_pk) = {
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
        vault_signer_pk,
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

pub struct ListingKeys {
    pub market_key: Keypair,
    pub req_q_key: Keypair,
    pub event_q_key: Keypair,
    pub bids_key: Keypair,
    pub asks_key: Keypair,
    pub vault_signer_pk: Pubkey,
    pub vault_signer_nonce: u64,
}

#[derive(Debug)]
pub struct MarketKeys {
    pub market_pk: Pubkey,
    pub req_q_pk: Pubkey,
    pub event_q_pk: Pubkey,
    pub bids_pk: Pubkey,
    pub asks_pk: Pubkey,
    pub coin_mint_pk: Pubkey,
    pub coin_vault_pk: Pubkey,
    pub coin_wallet_key: Keypair,
    pub pc_mint_pk: Pubkey,
    pub pc_vault_pk: Pubkey,
    pub pc_wallet_key: Keypair,
    pub vault_signer_pk: Pubkey,
}
