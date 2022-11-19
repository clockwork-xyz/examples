use {
    clockwork_sdk::client::{
        thread_program::{
            instruction::{thread_create, thread_pause},
            objects::Thread,
            objects::Trigger,
        },
        Client, ClientResult, SplToken,
    },
    solana_sdk::signer::Signer,
    solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Keypair},
};

pub mod instructions;
pub mod utils;

pub use instructions::*;
pub use utils::*;

fn main() -> ClientResult<()> {
    // let args: Vec<String> = env::args().collect();
    // let command: &str = &args[1];

    let payer = Keypair::new();
    let client = Client::new(payer, "http://localhost:8899".into());
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    let subscription_id = 4;
    let deposit_amount = 14000;

    let subscription =
        subscriptions_program::state::Subscription::pubkey(client.payer_pubkey(), subscription_id);

    let subscription_thread = Thread::pubkey(client.payer_pubkey(), "payment".into());

    let subscription_bank = subscriptions_program::state::Subscription::bank_pubkey(
        subscription,
        client.payer_pubkey(),
    );

    // create token mint
    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    let subscriber_token_account = client
        .create_token_account(&client.payer_pubkey(), &mint)
        .unwrap()
        .pubkey();

    let subscriber =
        subscriptions_program::state::Subscriber::pubkey(client.payer_pubkey(), subscription);

    client
        .mint_to(
            &client.payer,
            &mint,
            &subscriber_token_account,
            deposit_amount,
            9,
        )
        .unwrap();

    let recurrent_amount = 1500;
    let schedule = "0 * * ? * *".to_string();
    let is_active = true;

    create_subscription(
        &client,
        subscription_bank,
        mint,
        subscription,
        subscription_thread,
        recurrent_amount,
        schedule,
        is_active,
        subscription_id,
    )?;

    create_subscriber(&client, subscriber, subscription)?;

    create_queue(&client, subscriber, subscription, subscription_thread)?;

    // deposit(
    //     &client,
    //     subscriber,
    //     subscription,
    //     subscription_bank,
    //     subscriber_token_account,
    //     deposit_amount,
    // )?;

    Ok(())
}
