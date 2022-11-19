use {
    crate::*,
    anchor_lang::prelude::Pubkey,
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn create_queue(
    client: &Client,
    subscriber: Pubkey,
    subscription: Pubkey,
    subscription_thread: Pubkey,
) -> ClientResult<()> {
    // let create_queue_ix = Instruction {
    //     program_id: subscriptions_program::ID,
    //     accounts: vec![
    //         AccountMeta::new(client.payer_pubkey(), true),
    //         AccountMeta::new(subscriber, false),
    //         AccountMeta::new(subscription_queue, false),
    //         AccountMeta::new_readonly(subscription, false),
    //         AccountMeta::new_readonly(clockwork_crank::ID, false),
    //         AccountMeta::new_readonly(system_program::ID, false),
    //     ],
    //     data: subscriptions_program::instruction::CreateQueue {}.data(),
    // };

    // send_and_confirm_tx(
    //     client,
    //     [create_queue_ix].to_vec(),
    //     None,
    //     "create_queue".to_string(),
    // )?;

    let disburse_payment_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(subscriber, false),
            AccountMeta::new_readonly(subscription, false),
            AccountMeta::new_readonly(subscription_thread, false),
            AccountMeta::new_readonly(clockwork_crank::ID, false),
        ],
        data: clockwork_sdk::anchor_sighash("disburse_payment").into(),
    };

    let thread_create = thread_create(
        subscription,
        "payment".into(),
        disburse_payment_ix.into(),
        client.payer_pubkey(),
        subscription_thread,
        Trigger::Cron {
            schedule: "*/2 * * * * * *".into(),
            skippable: true,
        },
    );

    let thread_pause = thread_pause(subscription, subscription_thread);

    send_and_confirm_tx(
        &client,
        [thread_create].to_vec(),
        None,
        "create_thread and pause_thread".to_string(),
    )?;

    Ok(())
}
