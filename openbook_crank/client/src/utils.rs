use {
    anchor_lang::prelude::*,
    clockwork_client::{Client, ClientResult},
    solana_sdk::{instruction::Instruction, signature::Keypair, transaction::Transaction},
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
        Ok(sig) => println!(
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
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
