use {
    clockwork_sdk::client::SplToken,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::signer::Signer,
    solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair},
};

pub mod instructions;
pub mod utils;

pub use instructions::*;
pub use utils::*;

fn main() -> ClientResult<()> {
    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();
    let client = Client::new(payer, "http://localhost:8899".into());
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    let subscription_id = 4;

    let subscription =
        subscriptions_program::state::Subscription::pubkey(payer_pubkey, subscription_id);
    let subscription_queue =
        clockwork_crank::state::Queue::pubkey(subscription, "subscription".into());
    let subscription_bank = subscriptions_program::state::Subscription::bank_pubkey(
        subscription,
        client.payer_pubkey(),
    );

    // create token mint
    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    let recurrent_amount = 1500;
    let schedule = "0 * * ? * *".to_string();
    let is_active = true;

    create_subscription(
        &client,
        subscription_bank,
        mint,
        subscription,
        subscription_queue,
        recurrent_amount,
        schedule,
        is_active,
        subscription_id,
    )?;

    // let subscriber = clockwork_crank::state::Queue::pubkey(subscription, "subscription".into());

    // create_subscriber(&client, subscriber, subscription)?;

    Ok(())
}
