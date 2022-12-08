use {
    crate::*,
    anchor_lang::{prelude::Pubkey, InstructionData},
    clockwork_sdk::client::{Client, ClientResult},
    solana_sdk::instruction::{AccountMeta, Instruction},
};

pub fn update_auhority(
    client: &Client,
    subscription: Pubkey,
    new_authority: Pubkey,
) -> ClientResult<()> {
    let update_authority_ix = Instruction {
        program_id: subscriptions_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(subscription, false),
        ],
        data: subscriptions_program::instruction::UpdateAuthority { new_authority }.data(),
    };

    send_and_confirm_tx(
        client,
        [update_authority_ix].to_vec(),
        None,
        "update_auhority".to_string(),
    )?;

    Ok(())
}
