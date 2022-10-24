use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        system_program,
    },
};

pub fn create_subscriber(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
) -> ClientResult<()> {
    let create_subscriber_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscriber, false),
            AccountMeta::new_readonly(subscription, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: subscriptions_program::instruction::CreateSubscriber {}.data(),
    };

    send_and_confirm_tx(
        client,
        [create_subscriber_ix].to_vec(),
        None,
        "create_subscriber".to_string(),
    )?;

    Ok(())
}
