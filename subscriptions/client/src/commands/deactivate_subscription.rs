use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn deactivate_subscription(
    client: &Client,
    subscription: Pubkey,
    mint: Pubkey,
) -> ClientResult<()> {
    let deactivate_subscription_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscription, false),
            AccountMeta::new_readonly(mint, false),
        ],
        data: subscriptions_program::instruction::DeactivateSubscription {}.data(),
    };

    send_and_confirm_tx(
        client,
        [deactivate_subscription_ix].to_vec(),
        None,
        "deactivate_subscription".to_string(),
    )?;

    Ok(())
}
