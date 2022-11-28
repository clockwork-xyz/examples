use {
    clockwork_sdk::client::{thread_program::objects::Thread, ClientResult, SplToken},
    dotenv::dotenv,
    rand::Rng,
    solana_sdk::signer::Signer,
};

pub mod instructions;
pub mod utils;

pub use instructions::*;
pub use utils::*;

fn main() -> ClientResult<()> {
    // let args: Vec<String> = env::args().collect();
    // let command: &str = &args[1];
    dotenv().ok();
    let client = get_client();
    let mut rng = rand::thread_rng();

    let subscription_id = rng.gen::<u64>();
    let deposit_amount = 14000;
    let recurrent_amount = 1500;
    let schedule = "0 * * ? * *".to_string();
    let is_active = true;

    let (subscription, subscription_bump) =
        subscriptions_program::state::Subscription::pda(client.payer_pubkey(), subscription_id);

    let subscriber =
        subscriptions_program::state::Subscriber::pda(client.payer_pubkey(), subscription).0;

    let subscription_thread = Thread::pubkey(subscription, subscription_id.to_string());

    let (subscription_bank, _) =
        subscriptions_program::state::Subscription::bank_pda(subscription, client.payer_pubkey());

    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    // let subscriber_token_account = client
    //     .create_token_account(&client.payer_pubkey(), &mint)
    //     .unwrap()
    //     .pubkey();

    let subscriber_token_account = client
        .create_associated_token_account(&client.payer, &client.payer_pubkey(), &mint)
        .unwrap();

    client
        .mint_to(
            &client.payer,
            &mint,
            &subscriber_token_account,
            deposit_amount,
            9,
        )
        .unwrap();

    create_subscription(
        &client,
        subscription_bank,
        mint,
        subscription,
        recurrent_amount,
        schedule,
        is_active,
        subscription_id,
        subscription_bump,
    )?;

    create_subscriber(
        &client,
        subscriber,
        subscription,
        subscription_thread,
        subscriber_token_account,
        mint,
    )?;

    subscribe(
        &client,
        subscriber,
        subscription,
        subscriber_token_account,
        subscription_bank,
        mint,
    )?;

    // unsubscribe(&client, subscriber, subscription)?;

    print_config(
        subscription,
        subscription_thread,
        subscription_bank,
        subscriber,
        subscriber_token_account,
        mint,
        subscription_id,
    );

    Ok(())
}
