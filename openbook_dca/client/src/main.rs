mod utils;

use {
    anchor_lang::{prelude::*, solana_program::sysvar, system_program, InstructionData},
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

    // let sol_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6").unwrap(),
    //     event_q: Pubkey::from_str("8CvwxZ9Db6XbLD46NZwwmVDZZRDy7eydFcAGkXKh9axa").unwrap(),
    //     req_q: Pubkey::from_str("CPjXDcggXckEq9e4QeXUieVJBpUNpLEmpihLpg5vWjGF").unwrap(),
    //     bids: Pubkey::from_str("5jWUncPNBMZJ3sTHKmMLszypVkoRK6bfEQMQUHweeQnh").unwrap(),
    //     asks: Pubkey::from_str("EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5").unwrap(),
    //     coin_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), // Base/Coin/A/SOL
    //     coin_vault: Pubkey::from_str("CKxTHwM9fPMRRvZmFnFoqKNd9pQR21c5Aq9bh5h9oghX").unwrap(),
    //     pc_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // Quote/PC/B/USDC
    //     pc_vault: Pubkey::from_str("6A5NHCj1yF6urc9wZNe6Bcjj4LVszQNj5DwAWG97yzMu").unwrap(),
    // vault_signer: Pubkey::from_str("CTz5UMLQm2SRWHzQnU62Pi4yJqbNGjgRBHqqp6oDHfF7").unwrap(),
    // };

    let bonk_usdc_market_keys = MarketKeys {
        market: Pubkey::from_str("8PhnCfgqpgFM7ZJvttGdBVMXHuU4Q23ACxCvWkbs1M71").unwrap(),
        event_q: Pubkey::from_str("8BjvcgtwT5rdaypEMZFAA35gHeRT992aPFr4dwdcLf1v").unwrap(),
        req_q: Pubkey::from_str("DoPUJXzG6Q7TwoxXJ1PtETqCUDfovpHLhwvj737KTmua").unwrap(),
        bids: Pubkey::from_str("5F2yj13thTvdTaNdMvxgsczPRXfbqVd7tr13bJSFg1W7").unwrap(),
        asks: Pubkey::from_str("GHsWvxp6KJ3Yr8HH5L4pbHkC1YMGk8FzMknsYo1kzzZv").unwrap(),
        coin_mint: Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap(), // Base/Coin/A/SOL
        coin_vault: Pubkey::from_str("A9yRKSx8SyqNdCtCMUgr6wDXUs1JmVFkVno6FcscSD6m").unwrap(),
        pc_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // Quote/PC/B/USDC
        pc_vault: Pubkey::from_str("D9dojzvwJGs4q3Cx8ytvD8kWVVZszoVKvPZEZ5D8PV1Y").unwrap(),
        vault_signer: Pubkey::from_str("3oQLKk1TyyXMT14p2i8p95v5jKqsQ5qzZGwxZCTnFC7p").unwrap(),
    };

    dca_create(&client, &bonk_usdc_market_keys, "BONK_USDC_DCA".into())?;

    // dca_delete(&client, &bonk_usdc_market_keys, "BONK_USDC_DCA".into())?;

    Ok(())
}

fn dca_create(
    client: &Client,
    market_keys: &MarketKeys,
    dca_thread_id: String,
) -> ClientResult<()> {
    let mut dca_open_orders_account_pubkey = None;
    init_dex_account(&client, &mut dca_open_orders_account_pubkey)?;

    let dca_pubkey = openbook_dca::state::Dca::pubkey(client.payer_pubkey(), market_keys.market);

    let dca_thread_pubkey = Thread::pubkey(client.payer_pubkey(), dca_thread_id.clone());

    let authority_coin_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &market_keys.coin_mint);

    let dca_coin_vault_pubkey = get_associated_token_address(&dca_pubkey, &market_keys.coin_mint);

    let authority_pc_vault_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &market_keys.pc_mint);

    let dca_pc_vault_pubkey = get_associated_token_address(&dca_pubkey, &market_keys.pc_mint);

    let dca_create_ix = Instruction {
        program_id: openbook_dca::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(authority_coin_vault_pubkey, false),
            AccountMeta::new(authority_pc_vault_pubkey, false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new(dca_pubkey, false),
            AccountMeta::new(dca_coin_vault_pubkey, false),
            AccountMeta::new(dca_pc_vault_pubkey, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new_readonly(market_keys.coin_mint, false),
            AccountMeta::new_readonly(market_keys.pc_mint, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            // REMAINING ACCOUNTS
            AccountMeta::new(dca_open_orders_account_pubkey.unwrap(), false),
        ],
        data: openbook_dca::instruction::DcaCreate {
            swap_amount: 1000000,
        }
        .data(),
    };

    // create thread to transfer & swap
    let thread_create_swap_ix = thread_create(
        client.payer_pubkey(),
        dca_thread_id,
        Instruction {
            program_id: openbook_dca::ID,
            accounts: vec![
                AccountMeta::new(authority_coin_vault_pubkey, false),
                AccountMeta::new(authority_pc_vault_pubkey, false),
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new_readonly(dca_pubkey, false),
                AccountMeta::new(dca_coin_vault_pubkey, false),
                AccountMeta::new(dca_pc_vault_pubkey, false),
                AccountMeta::new_readonly(dca_thread_pubkey, true),
                AccountMeta::new_readonly(sysvar::rent::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token::ID, false),
                // REMAINING ACCOUNTS
                AccountMeta::new(market_keys.market, false),
                AccountMeta::new(market_keys.event_q, false),
                AccountMeta::new(market_keys.req_q, false),
                AccountMeta::new(market_keys.bids, false),
                AccountMeta::new(market_keys.asks, false),
                AccountMeta::new(market_keys.coin_vault, false),
                AccountMeta::new(market_keys.pc_vault, false),
                AccountMeta::new(market_keys.vault_signer, false),
                AccountMeta::new(dca_open_orders_account_pubkey.unwrap(), false),
            ],
            data: openbook_dca::instruction::Swap {}.data(),
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
        print_explorer_link(
            dca_open_orders_account_pubkey.unwrap(),
            "dca OOs account 📂".into(),
        )?;
        print_explorer_link(
            dca_open_orders_account_pubkey.unwrap(),
            "dca open orders account 📂".into(),
        )?;
        print_explorer_link(dca_thread_pubkey, "dca thread 📂".into())?;
        print_explorer_link(dca_pc_vault_pubkey, "dca PC vault 💰".into())?;
        print_explorer_link(authority_pc_vault_pubkey, "authority PC vault 💰".into())?;
        print_explorer_link(dca_coin_vault_pubkey, "dca Coin vault 💰".into())?;
        print_explorer_link(
            authority_coin_vault_pubkey,
            "authority Coin vault 💰".into(),
        )?;
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
    market_keys: &MarketKeys,
    dca_thread_id: String,
) -> ClientResult<()> {
    let dca_pubkey = openbook_dca::state::Dca::pubkey(client.payer_pubkey(), market_keys.market);
    let dca_thread_pubkey = Thread::pubkey(client.payer_pubkey(), dca_thread_id.clone());

    let dcas_thread_delete_ix = thread_delete(
        client.payer_pubkey(),
        client.payer_pubkey(),
        dca_thread_pubkey,
    );

    let dca_delete_ix = Instruction {
        program_id: openbook_dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(client.payer_pubkey(), true),
            AccountMeta::new(client.payer_pubkey(), false),
            AccountMeta::new(dca_pubkey, false),
        ],
        data: openbook_dca::instruction::DcaDelete {}.data(),
    };

    sign_send_and_confirm_tx(
        &client,
        [dca_delete_ix, dcas_thread_delete_ix].to_vec(),
        None,
        "dca delete".to_string(),
    )?;

    Ok(())
}
