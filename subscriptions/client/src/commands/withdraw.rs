use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    anchor_spl::token,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn withdraw(
    client: &Client,
    payer_token_account: Pubkey,
    subscription_bank: Pubkey,
    subscription: Pubkey,
    mint: Pubkey,
) -> ClientResult<()> {
    let withdraw_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(payer_token_account, false),
            AccountMeta::new(subscription, false),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: subscriptions_program::instruction::Withdraw {}.data(),
    };

    send_and_confirm_tx(client, [withdraw_ix].to_vec(), None, "withdraw".to_string())?;

    Ok(())
}
