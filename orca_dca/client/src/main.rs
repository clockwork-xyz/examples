mod utils;

use {
    anchor_lang::{prelude::*, system_program, InstructionData},
    anchor_spl::{
        associated_token::{self, get_associated_token_address},
        token,
    },
    clockwork_client::{
        thread::{
            state::Thread,
            {
                instruction::{thread_create, thread_delete},
                state::Trigger,
            },
        },
        Client, ClientResult,
    },
    solana_sdk::{instruction::Instruction, system_instruction::transfer},
    std::str::FromStr,
    utils::*,
};

fn main() -> ClientResult<()> {
    let client = default_client();

    let samo_usdc_pool_params = PoolParams {
        address: Pubkey::from_str("Epvp7qMYAF21VVjacdB3VfKn6nnXQSF4rGYu8sD6Bkow").unwrap(),
        authority: Pubkey::from_str("AB4rTE2JiKFhnfynUQCovbW75CUxT9LxcJX2SDTbY9gy").unwrap(),
        pool_token_mint: Pubkey::from_str("6VK1ksrmYGMBWUUZfygGF8tHRGpNxQEWv8pfvzQHdyyc").unwrap(),
        fee_account: Pubkey::from_str("9U8UF7d8kBvsS25XoZnjmVQ9vGkP4BUnHJgfc615BvG1").unwrap(),
        pool_a_vault: Pubkey::from_str("7jwHW4Lw3nVaSJXskN5pUoKU6YB9RBVfZtGBp3VbR43U").unwrap(), // SAMO Vault
        pool_b_vault: Pubkey::from_str("G7Gqjxk9EaJMeFfoFTSy9WfH8uurgQkbNQCREWAc56DZ").unwrap(), // USDC Vault
        token_ids: (
            Pubkey::from_str("7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU").unwrap(), // SAMO
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // USDC
        ),
    };

    dca_create(&client, &samo_usdc_pool_params, "BONK_USDC_ORCA_DCA".into())?;

    // dca_delete(&client, &samo_usdc_pool_params, "BONK_USDC_ORCA_DCA".into())?;

    Ok(())
}

fn dca_create(
    client: &Client,
    pool_params: &PoolParams,
    dca_thread_id: String,
) -> ClientResult<()> {
    let dca_pubkey = orca_dca::state::Dca::pubkey(
        client.payer_pubkey(),
        pool_params.token_ids.1,
        pool_params.token_ids.0,
    );

    let dca_thread_pubkey = Thread::pubkey(client.payer_pubkey(), dca_thread_id.clone());

    // USDC vauls
    let authority_a_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &pool_params.token_ids.1);

    let dca_a_vault_pubkey = get_associated_token_address(&dca_pubkey, &pool_params.token_ids.1);

    // SAMO vaults
    let authority_b_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &pool_params.token_ids.0);

    let dca_b_vault_pubkey = get_associated_token_address(&dca_pubkey, &pool_params.token_ids.0);

    let dca_create_ix = Instruction {
        program_id: orca_dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(authority_a_vault_pubkey, false),
            AccountMeta::new_readonly(pool_params.token_ids.1, false),
            AccountMeta::new(authority_b_vault_pubkey, false),
            AccountMeta::new_readonly(pool_params.token_ids.0, false),
            AccountMeta::new(dca_pubkey, false),
            AccountMeta::new(dca_a_vault_pubkey, false),
            AccountMeta::new(dca_b_vault_pubkey, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: orca_dca::instruction::DcaCreate {
            amount_in: 10000,
            minimum_amount_out: 10000,
        }
        .data(),
    };

    // create thread to transfer & swap
    let thread_create_swap_ix = thread_create(
        client.payer_pubkey(),
        dca_thread_id,
        Instruction {
            program_id: orca_dca::ID,
            accounts: vec![
                AccountMeta::new(authority_a_vault_pubkey, false),
                AccountMeta::new(authority_b_vault_pubkey, false),
                AccountMeta::new_readonly(dca_pubkey, false),
                AccountMeta::new(dca_a_vault_pubkey, false),
                AccountMeta::new(dca_b_vault_pubkey, false),
                AccountMeta::new_readonly(dca_thread_pubkey, true),
                AccountMeta::new_readonly(
                    Pubkey::from_str("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP").unwrap(),
                    false,
                ),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token::ID, false),
                // REMAINING ACCOUNTS
                AccountMeta::new(pool_params.address, false),
                AccountMeta::new(pool_params.authority, false),
                AccountMeta::new(pool_params.pool_b_vault, false),
                AccountMeta::new(pool_params.pool_a_vault, false),
                AccountMeta::new(pool_params.pool_token_mint, false),
                AccountMeta::new(pool_params.fee_account, false),
            ],
            data: orca_dca::instruction::ProxySwap {}.data(),
        }
        .into(),
        client.payer_pubkey(),
        dca_thread_pubkey,
        Trigger::Cron {
            schedule: "0 */2 * * * *".into(),
            skippable: true,
        },
    );

    let fund_swap_thread_ix = transfer(&client.payer_pubkey(), &dca_thread_pubkey, 100000000);

    {
        print_explorer_link(dca_pubkey, "dca account 📂".into())?;
        print_explorer_link(dca_thread_pubkey, "dca thread 📂".into())?;
        print_explorer_link(dca_a_vault_pubkey, "dca mint A vault 💰".into())?;
        print_explorer_link(authority_a_vault_pubkey, "authority mint A vault 💰".into())?;
        print_explorer_link(dca_b_vault_pubkey, "dca mint B vault 💰".into())?;
        print_explorer_link(authority_b_vault_pubkey, "authority mint B vault 💰".into())?;
    }

    sign_send_and_confirm_tx(
        &client,
        [
            dca_create_ix, // initialize dca acc and approve token account authority
        ]
        .to_vec(),
        None,
        "dca create".to_string(),
    )?;

    sign_send_and_confirm_tx(
        &client,
        [
            thread_create_swap_ix, // on schedule: transfer & swap; transfer & swap; ...
            fund_swap_thread_ix,
        ]
        .to_vec(),
        Some(vec![client.payer()]),
        "swap thread create".to_string(),
    )?;

    Ok(())
}

pub fn dca_delete(
    client: &Client,
    pool_params: &PoolParams,
    dca_thread_id: String,
) -> ClientResult<()> {
    let dca_pubkey = orca_dca::state::Dca::pubkey(
        client.payer_pubkey(),
        pool_params.token_ids.1,
        pool_params.token_ids.0,
    );
    let dca_thread_pubkey = Thread::pubkey(client.payer_pubkey(), dca_thread_id.clone());

    let dcas_thread_delete_ix = thread_delete(
        client.payer_pubkey(),
        client.payer_pubkey(),
        dca_thread_pubkey,
    );

    let dca_delete_ix = Instruction {
        program_id: orca_dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(client.payer_pubkey(), true),
            AccountMeta::new(client.payer_pubkey(), false),
            AccountMeta::new(dca_pubkey, false),
        ],
        data: orca_dca::instruction::DcaDelete {}.data(),
    };

    sign_send_and_confirm_tx(
        &client,
        [dca_delete_ix, dcas_thread_delete_ix].to_vec(),
        None,
        "dca delete".to_string(),
    )?;

    Ok(())
}
