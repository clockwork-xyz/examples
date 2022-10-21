use {
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
    #[cfg(feature = "devnet")]
    let client = Client::new(payer, "https://api.devnet.solana.com".into());
    #[cfg(not(feature = "devnet"))]
    let client = Client::new(payer, "http://localhost:8899".into());
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    let subscription =
        subscriptions_program::state::Subscription::pubkey(payer_pubkey, "first".to_string());
    let subscriptions_queue =
        clockwork_crank::state::Queue::pubkey(subscription, "subscription".into());

    // create_subscription(client, subscription_bank, mint, subscription, subscription_queue, recurrent_amount, schedule, is_active, subscription_id)

    Ok(())
}
