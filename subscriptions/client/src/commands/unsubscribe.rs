use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn unsubscribe(client: &Client, subscriber: Pubkey, subscription: Pubkey) -> ClientResult<()> {
    let unsubscribe_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscriber, false),
            AccountMeta::new(subscription, false),
        ],
        data: subscriptions_program::instruction::Unsubscribe {}.data(),
    };

    send_and_confirm_tx(
        client,
        [unsubscribe_ix].to_vec(),
        None,
        "unsubscribe".to_string(),
    )?;

    Ok(())
}
