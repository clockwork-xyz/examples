mod utils;

use {
    anchor_lang::{prelude::*, solana_program::sysvar, system_program, InstructionData},
    anchor_spl::{associated_token, token},
    clockwork_client::{
        thread::{
            state::Thread,
            {instruction::thread_create, state::Trigger},
        },
        Client, ClientResult,
    },
    solana_sdk::instruction::Instruction,
    std::str::FromStr,
    utils::*,
};

fn main() -> ClientResult<()> {
    let client = default_client();

    // let sol_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6").unwrap(),
    //     event_q: Pubkey::from_str("8CvwxZ9Db6XbLD46NZwwmVDZZRDy7eydFcAGkXKh9axa").unwrap(),
    //     bids: Pubkey::from_str("5jWUncPNBMZJ3sTHKmMLszypVkoRK6bfEQMQUHweeQnh").unwrap(),
    //     asks: Pubkey::from_str("EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5").unwrap(),
    //     pc_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
    //     pc_vault: Pubkey::from_str("CKxTHwM9fPMRRvZmFnFoqKNd9pQR21c5Aq9bh5h9oghX").unwrap(),
    //     pc_wallet: Pubkey::from_str("9d7WcMvuk9pU5EnNbUDJzuNdsQjaiJo5G7rLFtAozp17").unwrap(),
    //     coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     coin_vault: Pubkey::from_str("6A5NHCj1yF6urc9wZNe6Bcjj4LVszQNj5DwAWG97yzMu").unwrap(),
    //     coin_wallet: Pubkey::from_str("FZnhkDzQeNPZb4VADuucxymVARRRWKxNDh4FsNSSSwAP").unwrap(),
    //     vault_signer: Pubkey::from_str("CTz5UMLQm2SRWHzQnU62Pi4yJqbNGjgRBHqqp6oDHfF7").unwrap(),
    // };

    let bonk_usdc_market_keys = MarketKeys {
        market: Pubkey::from_str("8PhnCfgqpgFM7ZJvttGdBVMXHuU4Q23ACxCvWkbs1M71").unwrap(),
        event_q: Pubkey::from_str("8BjvcgtwT5rdaypEMZFAA35gHeRT992aPFr4dwdcLf1v").unwrap(),
        req_q: Pubkey::from_str("DoPUJXzG6Q7TwoxXJ1PtETqCUDfovpHLhwvj737KTmua").unwrap(),
        bids: Pubkey::from_str("5F2yj13thTvdTaNdMvxgsczPRXfbqVd7tr13bJSFg1W7").unwrap(),
        asks: Pubkey::from_str("GHsWvxp6KJ3Yr8HH5L4pbHkC1YMGk8FzMknsYo1kzzZv").unwrap(),
        pc_mint: Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap(), // bonk
        pc_vault: Pubkey::from_str("A9yRKSx8SyqNdCtCMUgr6wDXUs1JmVFkVno6FcscSD6m").unwrap(),
        pc_wallet: Pubkey::from_str("Fuv3U8c1nuhRWdmPcptBqHTq7Cshb8YKtVeiS5BR6YSJ").unwrap(),
        coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // usdc
        coin_vault: Pubkey::from_str("D9dojzvwJGs4q3Cx8ytvD8kWVVZszoVKvPZEZ5D8PV1Y").unwrap(),
        coin_wallet: Pubkey::from_str("DtFi8ZBMoTSURfYfvyViWRMipXbnaLNhzbps5JiowtE8").unwrap(),
        vault_signer: Pubkey::from_str("3oQLKk1TyyXMT14p2i8p95v5jKqsQ5qzZGwxZCTnFC7p").unwrap(),
    };

    let investment_pubkey = investments_program::state::Investment::pubkey(
        client.payer_pubkey(),
        bonk_usdc_market_keys.market,
    );
    let investment_thread_pubkey = Thread::pubkey(client.payer_pubkey(), "investment".to_string());
    let settle_funds_thread_pubkey =
        Thread::pubkey(client.payer_pubkey(), "settle_funds".to_string());

    let mut investment_open_orders_account_pubkey = None;

    init_dex_account(&client, &mut investment_open_orders_account_pubkey)?;

    let authority_mint_a_vault_pubkey = anchor_spl::associated_token::get_associated_token_address(
        &client.payer_pubkey(),
        &bonk_usdc_market_keys.pc_mint,
    );

    let investment_mint_a_vault_pubkey = anchor_spl::associated_token::get_associated_token_address(
        &investment_pubkey,
        &bonk_usdc_market_keys.pc_mint,
    );

    investment_create(
        &client,
        investment_pubkey,
        investment_thread_pubkey,
        settle_funds_thread_pubkey,
        authority_mint_a_vault_pubkey,
        investment_mint_a_vault_pubkey,
        &mut investment_open_orders_account_pubkey,
        &bonk_usdc_market_keys,
    )?;

    Ok(())
}

fn investment_create(
    client: &Client,
    investment_pubkey: Pubkey,
    investment_thread_pubkey: Pubkey,
    settle_funds_thread_pubkey: Pubkey,
    authority_mint_a_vault_pubkey: Pubkey,
    investment_mint_a_vault_pubkey: Pubkey,
    investment_open_orders_account_pubkey: &mut Option<Pubkey>,
    market_keys: &MarketKeys,
) -> ClientResult<()> {
    init_dex_account(&client, investment_open_orders_account_pubkey)?;

    let investment_create_ix = Instruction {
        program_id: investments_program::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(authority_mint_a_vault_pubkey, false),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new(investment_pubkey, false),
            AccountMeta::new(investment_mint_a_vault_pubkey, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new_readonly(market_keys.pc_mint, false),
            AccountMeta::new_readonly(market_keys.coin_mint, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: investments_program::instruction::InvestmentCreate {
            swap_amount: 000_001,
        }
        .data(),
    };

    // create thread with read events ix
    let thread_create_deposit_ix = thread_create(
        client.payer_pubkey(),
        "investment".into(),
        Instruction {
            program_id: investments_program::ID,
            accounts: vec![
                AccountMeta::new(authority_mint_a_vault_pubkey, false),
                AccountMeta::new_readonly(investment_pubkey, false),
                AccountMeta::new(investment_mint_a_vault_pubkey, false),
                AccountMeta::new_readonly(investment_thread_pubkey, true),
                AccountMeta::new(market_keys.market, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token::ID, false),
                // REMAINING ACCOUNTS
                AccountMeta::new_readonly(market_keys.event_q, false),
                AccountMeta::new_readonly(market_keys.req_q, false),
                AccountMeta::new_readonly(market_keys.bids, false),
                AccountMeta::new_readonly(market_keys.asks, false),
                AccountMeta::new_readonly(market_keys.pc_vault, false),
                AccountMeta::new_readonly(market_keys.coin_vault, false),
                AccountMeta::new_readonly(investment_open_orders_account_pubkey.unwrap(), false),
            ],
            data: investments_program::instruction::Deposit {}.data(),
        }
        .into(),
        client.payer_pubkey(),
        settle_funds_thread_pubkey,
        Trigger::Cron {
            schedule: "0 */2 * * * *".into(),
            skippable: true,
        },
    );

    // create thread with read events ix
    let thread_create_settle_funds_ix = thread_create(
        client.payer_pubkey(),
        "settle_funds".into(),
        Instruction {
            program_id: investments_program::ID,
            accounts: vec![
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new_readonly(investment_pubkey, false),
                AccountMeta::new_readonly(investment_thread_pubkey, true),
                AccountMeta::new_readonly(market_keys.market, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(token::ID, false),
                // REMAINING ACCOUNTS
                AccountMeta::new_readonly(investment_open_orders_account_pubkey.unwrap(), false),
                AccountMeta::new_readonly(market_keys.coin_vault, false),
                AccountMeta::new_readonly(market_keys.coin_wallet, false),
                AccountMeta::new_readonly(market_keys.pc_vault, false),
                AccountMeta::new_readonly(market_keys.pc_wallet, false),
                AccountMeta::new_readonly(market_keys.vault_signer, false),
            ],
            data: investments_program::instruction::SettleFunds {}.data(),
        }
        .into(),
        client.payer_pubkey(),
        investment_thread_pubkey,
        Trigger::Account {
            address: investment_open_orders_account_pubkey.unwrap(),
            offset: 20,
            size: 20,
        },
    );

    sign_send_and_confirm_tx(
        &client,
        [
            investment_create_ix, // initialize investment acc and approve token account authority
            thread_create_deposit_ix, // investment deposit -> swap -> deposit -> ...
            thread_create_settle_funds_ix, // on open order account state change: settle_funds -> claim -> settle_funds -> ...
        ]
        .to_vec(),
        None,
        "investment create, deposit/swap thread create, settle_funds/claim thread create"
            .to_string(),
    )?;

    Ok(())
}
