use {
    crate::*,
    anchor_lang::prelude::Pubkey,
    clockwork_sdk::client::{
        thread_program::{
            self,
            instruction::{thread_create, thread_pause},
        },
        Client, ClientResult,
    },
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn create_queue(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
    subscription_thread: Pubkey,
) -> ClientResult<()> {
    let disburse_payment_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(subscriber, false),
            AccountMeta::new(subscription, false),
            AccountMeta::new(subscription_thread, false),
            AccountMeta::new_readonly(thread_program::ID, false),
        ],
        data: clockwork_sdk::anchor_sighash("disburse_payment").into(),
    };

    let thread_create = thread_create(
        client.payer_pubkey(),
        "payment".into(),
        disburse_payment_ix.into(),
        client.payer_pubkey(),
        subscription_thread,
        Trigger::Cron {
            schedule: "*/2 * * * * * *".into(),
            skippable: true,
        },
    );

    let thread_pause = thread_pause(client.payer_pubkey(), subscription_thread);

    send_and_confirm_tx(
        client,
        [thread_create, thread_pause].to_vec(),
        None,
        "create_thread and pause_thread".to_string(),
    )?;

    Ok(())
}
