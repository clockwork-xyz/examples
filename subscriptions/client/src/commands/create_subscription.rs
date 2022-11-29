use {
    crate::*,
    anchor_lang::{prelude::Pubkey, solana_program::sysvar, InstructionData},
    anchor_spl::token,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        system_program,
    },
};

pub fn create_subscription(
    client: &Client,
    subscription_bank: Pubkey,
    mint: Pubkey,
    subscription: Pubkey,
    recurrent_amount: u64,
    schedule: String,
    is_active: bool,
    subscription_id: u64,
    bump: u8,
) -> ClientResult<()> {
    let create_subscription_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(subscription, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: subscriptions_program::instruction::CreateSubscription {
            recurrent_amount,
            schedule,
            mint,
            is_active,
            subscription_id,
            bump,
        }
        .data(),
    };

    send_and_confirm_tx(
        client,
        [create_subscription_ix].to_vec(),
        None,
        "create_subscription".to_string(),
    )?;
    println!("- - - - - - - - - - UPDATE YOUR .ENV FILE - - - - - - - - - -");
    println!("SUBSCRIPTION=\"{:?}\"", subscription);
    println!("SUBSCRIPTION_BANK=\"{:?}\"", subscription_bank);
    println!("SUBSCRIPTION_ID=\"{:?}\"", subscription_id);

    Ok(())
}
