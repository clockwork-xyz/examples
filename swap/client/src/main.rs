mod utils;

use {
    anchor_lang::{prelude::*, system_program, InstructionData},
    anchor_spl::{associated_token::get_associated_token_address, token},
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
    spl_associated_token_account::instruction::create_associated_token_account,
    std::str::FromStr,
    utils::*,
    whirlpool::{utils::get_tick_array_pubkeys, TickArray},
};

fn main() -> ClientResult<()> {
    let client = default_client();

    let bonk_usdc_whirlpool = WhirlpoolParams {
        whirlpool: Pubkey::from_str("8QaXeHBrShJTdtN1rWCccBxpSVvKksQ2PCu5nufb2zbk").unwrap(),
        token_mint_a: Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap(), // BONK
        token_a_decimals: 5,
        token_mint_b: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // USDC
        token_b_decimals: 6,
        oracle: Pubkey::from_str("4QqfXtmcMfHAQstgVuhDqY1UyHzyiBfwMrz7Jbgt8aQL").unwrap(),
    };

    #[cfg(not(feature = "delete"))]
    dca_create(
        &client,
        &bonk_usdc_whirlpool,
        "BONK_USDC_WP_DCA".into(),
        1000000,
        true,
    )?;

    #[cfg(feature = "delete")]
    swap_delete(&client, "BONK_USDC_WP_DCA".into())?;

    Ok(())
}

fn dca_create(
    client: &Client,
    whirlpool_params: &WhirlpoolParams,
    swap_thread_id: String,
    amount: u64,
    a_to_b: bool,
) -> ClientResult<()> {
    let whirlpool_id = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc").unwrap();

    let swap_thread_pubkey = Thread::pubkey(client.payer_pubkey(), swap_thread_id.clone());

    // BONK vaults
    let authority_a_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &whirlpool_params.token_mint_a);

    let swap_thread_a_vault_pubkey =
        get_associated_token_address(&swap_thread_pubkey, &whirlpool_params.token_mint_a);

    // USDC vaults
    let authority_b_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &whirlpool_params.token_mint_b);

    let swap_thread_b_vault_pubkey =
        get_associated_token_address(&swap_thread_pubkey, &whirlpool_params.token_mint_b);

    let token_acc_pubkeys = vec![
        (
            &authority_a_vault_pubkey,
            &whirlpool_params.token_mint_a,
            client.payer_pubkey(),
        ),
        (
            &authority_b_vault_pubkey,
            &whirlpool_params.token_mint_b,
            client.payer_pubkey(),
        ),
        (
            &swap_thread_a_vault_pubkey,
            &whirlpool_params.token_mint_a,
            swap_thread_pubkey,
        ),
        (
            &swap_thread_b_vault_pubkey,
            &whirlpool_params.token_mint_b,
            swap_thread_pubkey,
        ),
    ];

    let mut init_ata_ixs = vec![];

    // create init ATA ix if it doesn't already exist
    for (ata, mint, owner) in token_acc_pubkeys {
        match client.get_account_data(&ata) {
            Ok(_data) => {}
            Err(_) => {
                init_ata_ixs.push(create_associated_token_account(
                    &client.payer_pubkey(),
                    &owner,
                    mint,
                    &token::ID,
                ));
            }
        }
    }

    let mut whirlpool_data: &[u8] = &client
        .get_account_data(&whirlpool_params.whirlpool)
        .unwrap();
    let whirlpool_state =
        whirlpool::state::Whirlpool::try_deserialize(&mut whirlpool_data).unwrap();

    let tick_array_pubkeys = get_tick_array_pubkeys(
        whirlpool_state.tick_current_index,
        whirlpool_state.tick_spacing,
        a_to_b,
        &whirlpool_id,
        &whirlpool_params.whirlpool,
    );

    let mut tick_arrays: Vec<TickArray> = Vec::with_capacity(3);

    for i in 0..3 {
        match client.get_account_data(&tick_array_pubkeys[i]) {
            Ok(data) => {
                tick_arrays
                    .push(whirlpool::TickArray::try_deserialize(&mut data.as_slice()).unwrap());
            }
            Err(_) => {}
        }
    }

    // create thread to transfer & swap
    let thread_create_swap_ix = thread_create(
        client.payer_pubkey(),
        swap_thread_id,
        Instruction {
            program_id: swap::ID,
            accounts: swap::accounts::OrcaWhirlpoolPreSwap {
                a_mint: whirlpool_state.token_mint_a,
                b_mint: whirlpool_state.token_mint_b,
                authority_a_vault: authority_a_vault_pubkey,
                authority_b_vault: authority_b_vault_pubkey,
                swap_thread: swap_thread_pubkey,
                swap_thread_a_vault: swap_thread_a_vault_pubkey,
                swap_thread_b_vault: swap_thread_b_vault_pubkey,
                oracle: whirlpool_params.oracle,
                system_program: system_program::ID,
                token_program: token::ID,
                whirlpool: whirlpool_params.whirlpool,
                orca_whirlpool_program: whirlpool_id,
                whirlpool_token_a_vault: whirlpool_state.token_vault_a,
                whirlpool_token_b_vault: whirlpool_state.token_vault_b,
            }
            .to_account_metas(Some(true)),
            data: swap::instruction::OrcaWhirlpoolPreswap { amount, a_to_b }.data(),
        }
        .into(),
        client.payer_pubkey(),
        swap_thread_pubkey,
        Trigger::Cron {
            schedule: "*/15 * * * * *".into(),
            skippable: true,
        },
    );

    let fund_swap_thread_ix = transfer(&client.payer_pubkey(), &swap_thread_pubkey, 100000000);

    let approve_token_delegation_ix = anchor_spl::token::spl_token::instruction::approve(
        &token::ID,
        if a_to_b {
            &authority_a_vault_pubkey
        } else {
            &authority_b_vault_pubkey
        },
        &swap_thread_pubkey,
        &client.payer_pubkey(),
        &[&client.payer_pubkey()],
        u64::MAX,
    )
    .unwrap();

    {
        print_explorer_link(swap_thread_pubkey, "swap thread 📂".into())?;
        print_explorer_link(
            swap_thread_a_vault_pubkey,
            "swap thread mint A vault 💰 (BONK)".into(),
        )?;
        print_explorer_link(
            authority_a_vault_pubkey,
            "authority mint A vault 💰 (BONK)".into(),
        )?;
        print_explorer_link(
            swap_thread_b_vault_pubkey,
            "swap thread mint B vault 💰 (USDC)".into(),
        )?;
        print_explorer_link(
            authority_b_vault_pubkey,
            "authority mint B vault 💰 (USDC)".into(),
        )?;
        print_explorer_link(
            whirlpool_params.token_mint_a,
            "whirlpool token A mint 🪙 ".into(),
        )?;
        print_explorer_link(
            whirlpool_params.token_mint_b,
            "whirlpool token B mint 🪙 ".into(),
        )?;
    }

    sign_send_and_confirm_tx(
        &client,
        [
            vec![
                thread_create_swap_ix,
                fund_swap_thread_ix,
                approve_token_delegation_ix,
            ],
            init_ata_ixs,
        ]
        .concat(),
        Some(vec![client.payer()]),
        "stateless swap - thread create, fund thread, approve TA, init ATAs".to_string(),
    )?;

    Ok(())
}

pub fn swap_delete(client: &Client, swap_thread_id: String) -> ClientResult<()> {
    let swap_thread_pubkey = Thread::pubkey(client.payer_pubkey(), swap_thread_id.clone());

    let swap_thread_delete_ix = thread_delete(
        client.payer_pubkey(),
        client.payer_pubkey(),
        swap_thread_pubkey,
    );

    sign_send_and_confirm_tx(
        &client,
        [swap_thread_delete_ix].to_vec(),
        None,
        "dca delete".to_string(),
    )?;

    Ok(())
}
