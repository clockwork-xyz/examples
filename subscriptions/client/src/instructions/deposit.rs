use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    anchor_spl::token,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn deposit(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
    subscription_bank: Pubkey,
    subscriber_token_account: Pubkey,
) -> ClientResult<()> {
    let deposit_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscriber, false),
            AccountMeta::new(subscriber_token_account, false),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new_readonly(subscription, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: subscriptions_program::instruction::CreateQueue {}.data(),
    };

    send_and_confirm_tx(client, [deposit_ix].to_vec(), None, "deposit".to_string())?;

    Ok(())
}
