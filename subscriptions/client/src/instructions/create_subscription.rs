use {
    anchor_lang::{prelude::Pubkey, solana_program::sysvar, InstructionData},
    anchor_spl::{associated_token, token},
    solana_client_helpers::{Client, ClientResult, RpcClient, SplToken},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    },
};

//  pub rent: Sysvar<'info, Rent>,

fn create_subscription(
    client: &Client,
    subscription_bank: Pubkey,
    mint: Pubkey,
    subscription: Pubkey,
    subscription_queue: Pubkey,
    recurrent_amount: u64,
    schedule: String,
    is_active: bool,
    subscription_id: String,
) -> ClientResult<()> {
    let create_payment_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscription_bank, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(subscription, false),
            AccountMeta::new(subscription_queue, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(clockwork_crank::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: subscriptions_program::instruction::CreateSubscription {
            recurrent_amount: 12,
            schedule: "12".to_string(),
            mint,
            is_active: true,
            subscription_id: "12".to_string(),
        }
        .data(),
    };

    // let top_up_payment_ix = Instruction {
    //     program_id: payments_program::ID,
    //     accounts: vec![
    //         AccountMeta::new_readonly(associated_token::ID, false),
    //         AccountMeta::new(escrow, false),
    //         AccountMeta::new(payment, false),
    //         AccountMeta::new_readonly(mint, false),
    //         AccountMeta::new_readonly(recipient, false),
    //         AccountMeta::new_readonly(sysvar::rent::ID, false),
    //         AccountMeta::new(client.payer_pubkey(), true),
    //         AccountMeta::new(sender_token_account, false),
    //         AccountMeta::new_readonly(system_program::ID, false),
    //         AccountMeta::new_readonly(token::ID, false),
    //     ],
    //     data: payments_program::instruction::TopUpPayment {
    //         amount: LAMPORTS_PER_SOL,
    //     }
    //     .data(),
    // };

    // send_and_confirm_tx(
    //     client,
    //     &[create_payment_ix, top_up_payment_ix],
    //     "create_payment_with_top_up".to_string(),
    // )?;
    //
    //     println!(
    //         "queue: https://explorer.solana.com/address/{}?cluster=custom",
    //         payment_queue
    //     );
    Ok(())
}
