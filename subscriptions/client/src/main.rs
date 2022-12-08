use {
    anchor_lang::prelude::Pubkey,
    clap::Parser,
    clockwork_sdk::client::{thread_program::objects::Thread, ClientResult, SplToken},
    dotenv::dotenv,
    rand::Rng,
    solana_sdk::signer::Signer,
};

pub mod commands;
pub mod utils;

pub use commands::*;
pub use utils::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute
    #[arg(short, long)]
    command: String,
    #[arg(short, long, default_value_t = 1000)]
    recurrent_amount: u64,
}

fn main() -> ClientResult<()> {
    let args = Args::parse();
    dotenv().ok();

    let (
        client,
        subscription,
        subscription_thread,
        subscription_bank,
        subscriber,
        subscriber_token_account,
        mint,
        subscription_id,
    ) = get_env_vars();

    match args.command.as_str() {
        "create_mint" => create_mint(&client)?,
        "create_subscription" => {
            let mut rng = rand::thread_rng();
            let subscription_id = rng.gen::<u64>();
            let schedule = "*/10 * * * * * *".to_string();
            let is_active = true;

            let (subscription, subscription_bump) = subscriptions_program::state::Subscription::pda(
                client.payer_pubkey(),
                subscription_id,
            );
            let (subscription_bank, _) = subscriptions_program::state::Subscription::bank_pda(
                subscription,
                client.payer_pubkey(),
            );

            create_subscription(
                &client,
                subscription_bank,
                mint.unwrap(),
                subscription,
                args.recurrent_amount,
                schedule,
                is_active,
                subscription_id,
                subscription_bump,
            )?;
        }
        "create_subscriber" => {
            let (subscriber, subscriber_bump) = subscriptions_program::state::Subscriber::pda(
                client.payer_pubkey(),
                subscription.unwrap(),
            );

            let subscription_thread = Thread::pubkey(subscriber, "subscriber_thread".to_string());

            create_subscriber(
                &client,
                subscriber,
                subscription.unwrap(),
                subscription_thread,
                subscriber_token_account.unwrap(),
                mint.unwrap(),
                subscription_bank.unwrap(),
                subscriber_bump,
            )?;
        }
        "subscribe" => {
            subscribe(
                &client,
                subscriber.unwrap(),
                subscription.unwrap(),
                subscriber_token_account.unwrap(),
                subscription_bank.unwrap(),
                mint.unwrap(),
                subscription_thread.unwrap(),
            )?;
        }
        "unsubscribe" => {
            unsubscribe(&client, subscriber.unwrap(), subscription.unwrap())?;
        }
        "deactivate_subscription" => {
            deactivate_subscription(&client, subscription.unwrap(), mint.unwrap())?;
        }
        "update_authority" => {
            let new_authority = Pubkey::new_unique();
            update_auhority(&client, subscription.unwrap(), new_authority)?;
        }
        "withdraw" => {
            withdraw(
                &client,
                subscriber_token_account.unwrap(),
                subscription_bank.unwrap(),
                subscription.unwrap(),
                mint.unwrap(),
            )?;
        }
        _ => {
            println!("Available Commands");
            println!("cargo run -- --command create_mint");
            println!("cargo run -- --command create_subscription --recurrent_amount <amount>");
            println!("cargo run -- --command deactivate_subscription");
            println!("cargo run -- --command withdraw");
            println!("cargo run -- --command create_subscriber");
            println!("cargo run -- --command subscribe");
            println!("cargo run -- --command unsubscribe");
        }
    };

    Ok(())
}
