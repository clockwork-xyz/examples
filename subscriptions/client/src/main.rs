use {
    solana_client_helpers::{Client, ClientResult, RpcClient},
    solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer},
};

pub mod utils;
pub use utils::*;

pub mod instructions;
pub use instructions::*;

fn main() -> ClientResult<()> {
    #[cfg(feature = "devnet")]
    let client = RpcClient::new("https://api.devnet.solana.com");

    let payer = Keypair::new();
    let payer_pubkey = payer.pubkey();
    let client = Client {
        client: RpcClient::new("http://localhost:8899"),
        payer,
    };
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    let subscription =
        subscriptions_program::state::Subscription::pubkey(payer_pubkey, "first".to_string());
    let subscriptions_queue =
        clockwork_crank::state::Queue::pubkey(subscription, "subscription".into());

    client.airdrop(&subscriptions_queue, LAMPORTS_PER_SOL)?;

    Ok(())
}
