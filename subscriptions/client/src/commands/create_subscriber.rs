use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    anchor_spl::token,
    clockwork_sdk::client::{Client, ClientResult},
    clockwork_sdk::thread_program,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        system_program,
    },
};

pub fn create_subscriber(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
    subscription_thread: Pubkey,
    subscriber_token_account: Pubkey,
    mint: Pubkey,
    subscription_bank: Pubkey,
    subscriber_bump: u8,
) -> ClientResult<()> {
    let create_subscriber_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscriber, false),
            AccountMeta::new(subscriber_token_account, false),
            AccountMeta::new_readonly(subscription, false),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new(subscription_thread, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(thread_program::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: subscriptions_program::instruction::CreateSubscriber { subscriber_bump }.data(),
    };

    send_and_confirm_tx(
        client,
        [create_subscriber_ix].to_vec(),
        None,
        "create_subscriber".to_string(),
    )?;
    println!("- - - - - - - - - - UPDATE YOUR .ENV FILE - - - - - - - - - -");
    println!("SUBSCRIPTION_THREAD=\"{:?}\"", subscription_thread);
    println!("SUBSCRIBER=\"{:?}\"", subscriber);

    Ok(())
}
