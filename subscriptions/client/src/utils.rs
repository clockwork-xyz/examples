use {
    solana_client_helpers::{Client, ClientResult},
    solana_sdk::{instruction::Instruction, transaction::Transaction},
};

pub fn send_and_confirm_tx(client: &Client, ix: &[Instruction], label: String) -> ClientResult<()> {
    // Create tx
    let mut tx = Transaction::new_with_payer(ix, Some(&client.payer_pubkey()));
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
