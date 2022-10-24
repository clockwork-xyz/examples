use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    anchor_spl::token,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn withdraw(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
    subscription_bank: Pubkey,
    subscriber_token_account: Pubkey,
    amount: u64,
) -> ClientResult<()> {
    let withdraw_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscriber, false),
            AccountMeta::new(subscriber_token_account, false),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new_readonly(subscription, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: subscriptions_program::instruction::Withdraw { amount }.data(),
    };

    send_and_confirm_tx(client, [withdraw_ix].to_vec(), None, "withdraw".to_string())?;

    Ok(())
}
