use clockwork_client::thread::instruction::thread_delete;

mod utils;

use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_client::{
        thread::{instruction::thread_create, state::Trigger},
        Client, ClientResult,
    },
    solana_sdk::{instruction::Instruction, system_instruction::transfer},
    std::str::FromStr,
    utils::*,
};

fn main() -> ClientResult<()> {
    let client = default_client();

    let sol_usdc_market_keys = MarketKeys {
        market: Pubkey::from_str("8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6").unwrap(),
        event_q: Pubkey::from_str("8CvwxZ9Db6XbLD46NZwwmVDZZRDy7eydFcAGkXKh9axa").unwrap(),
        bids: Pubkey::from_str("5jWUncPNBMZJ3sTHKmMLszypVkoRK6bfEQMQUHweeQnh").unwrap(),
        asks: Pubkey::from_str("EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5").unwrap(),
        coin_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(), // Base/Coin/A/SOL
        coin_vault: Pubkey::from_str("CKxTHwM9fPMRRvZmFnFoqKNd9pQR21c5Aq9bh5h9oghX").unwrap(),
        coin_wallet: Pubkey::from_str("9d7WcMvuk9pU5EnNbUDJzuNdsQjaiJo5G7rLFtAozp17").unwrap(),
        pc_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // Quote/PC/B/USDC
        pc_vault: Pubkey::from_str("6A5NHCj1yF6urc9wZNe6Bcjj4LVszQNj5DwAWG97yzMu").unwrap(),
        pc_wallet: Pubkey::from_str("FZnhkDzQeNPZb4VADuucxymVARRRWKxNDh4FsNSSSwAP").unwrap(),
        vault_signer: Pubkey::from_str("CTz5UMLQm2SRWHzQnU62Pi4yJqbNGjgRBHqqp6oDHfF7").unwrap(),
    };

    // let doggo_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("9fD2u4PbBoN8y3vvAtLMpVDFw2ThPWA11PV6CcsiSnu5").unwrap(),
    //     event_q: Pubkey::from_str("Fa4mRrRPTEbkW8hs1ER3EKdtxqxGiD63JuL7Dk2Ew7g8").unwrap(),
    //     bids: Pubkey::from_str("AwEDEUgZP9nN8nvS7zEfWggLFf2k8obc6jsem6Jedfmh").unwrap(),
    //     asks: Pubkey::from_str("4ABNf3dCWQg6NN5Xc1NXWN7m5vm1zutFe5RSXhSKn5KR").unwrap(),
    //     coin_mint: Pubkey::from_str("Doggoyb1uHFJGFdHhJf8FKEBUMv58qo98CisWgeD7Ftk").unwrap(),
    //     coin_vault: Pubkey::from_str("2P6WAjDg6ibRjNDYatmLow9feCW85tJcgZyaZnGi8Rxo").unwrap(),
    //     coin_wallet: Pubkey::from_str("BJKQiDRtVU5ecJ7UzNSmBCUmNFWqSoyK5vSS4RynnDhf").unwrap(),
    //     pc_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     pc_vault: Pubkey::from_str("C6fNmJmvbjiU24Nc6GtFA85s6YhRjzTdz9o671EVKkqF").unwrap(),
    //     pc_wallet: Pubkey::from_str("27VkqLYvBKfnosJcvfPbwqomVsVZA81LF5pYb4C36MLU").unwrap(),
    //     vault_signer: Pubkey::from_str("DnVM9L2RKH9xMnbHbNt239PVxcFFWTsTb2aKu9aATqZc").unwrap(),
    // };

    // let basis_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("FfP1cFGHeUfJmJKWhEvA8eUArCQvVgVHodt2AfLdWMdf").unwrap(),
    //     event_q: Pubkey::from_str("DPabHqDzAWN4fEzxY4Wfa4aLuRLmGyUAFAzCcAnauUiG").unwrap(),
    //     bids: Pubkey::from_str("2ZcywQ1xA37hLYpaDW9a6J6LxyTxjvJf7b7nBfxutbL2").unwrap(),
    //     asks: Pubkey::from_str("CQ5MQBXS6gfcdkJNQujU4viqrdpvN85A2Fa8qERCMSrY").unwrap(),
    //     coin_mint: Pubkey::from_str("Basis9oJw9j8cw53oMV7iqsgo6ihi9ALw4QR31rcjUJa").unwrap(),
    //     coin_vault: Pubkey::from_str("jrLgmkQ3ZyGvv2rxaMvg7i7QVwzTD5Co8LVWE1gSqrs").unwrap(),
    //     coin_wallet: Pubkey::from_str("8tZ33NZ6geKLrcN6yVrdLGbsY3bmjJ51kKCNtJaU7xE1").unwrap(),
    //     pc_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     pc_vault: Pubkey::from_str("GpFgz9Pjf6MqW5CNxqWzmSxhpXpUyL32khUbLpQ3X968").unwrap(),
    //     pc_wallet: Pubkey::from_str("5AZdzEh5qvexHgJYCKXZD6LRDLMj2L7EWQETM3nyu9UE").unwrap(),
    //     vault_signer: Pubkey::from_str("JCrdR5BFUZCd4ZBWTNVFP95PsHGn4QHyn5sw3s4QnJ3X").unwrap(),
    // };

    initialize_openbook_crank(
        &client,
        &sol_usdc_market_keys,
        "SOL_USDC_OPENBOOK_CRANK".into(),
    )?;

    // crank_delete(
    //     &client,
    //     &sol_usdc_market_keys,
    //     "SOL_USDC_OPENBOOK_CRANK".into(),
    // )?;

    Ok(())
}

fn initialize_openbook_crank(
    client: &Client,
    market_keys: &MarketKeys,
    id: String,
) -> ClientResult<()> {
    let crank_pubkey =
        openbook_crank::state::Crank::pubkey(client.payer_pubkey(), market_keys.market);
    let crank_thread_pubkey =
        clockwork_client::thread::state::Thread::pubkey(client.payer_pubkey(), id.clone());

    print_explorer_link(crank_thread_pubkey, "crank_thread".into())?;

    // define initialize ix
    let initialize_ix = Instruction {
        program_id: openbook_crank::ID,
        accounts: vec![
            AccountMeta::new(crank_pubkey, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new_readonly(market_keys.event_q, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: openbook_crank::instruction::Initialize {}.data(),
    };

    // create thread with consume events ix
    let crank_thread_create = thread_create(
        client.payer_pubkey(),
        id,
        Instruction {
            program_id: openbook_crank::ID,
            accounts: vec![
                AccountMeta::new_readonly(crank_pubkey, false),
                AccountMeta::new_readonly(crank_thread_pubkey, true),
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new(market_keys.event_q, false),
                AccountMeta::new(market_keys.market, false),
                AccountMeta::new(market_keys.coin_vault, false),
                AccountMeta::new(market_keys.pc_vault, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: openbook_crank::instruction::ConsumeEvents.data(),
        }
        .into(),
        client.payer_pubkey(),
        crank_thread_pubkey,
        Trigger::Account {
            address: market_keys.event_q,
            offset: 8 + 8,
            size: 8,
        },
    );

    let fund_consume_events_thread_ix =
        transfer(&client.payer_pubkey(), &crank_thread_pubkey, 100000000);

    sign_send_and_confirm_tx(
        client,
        vec![
            initialize_ix,
            crank_thread_create,
            fund_consume_events_thread_ix,
        ],
        None,
        "initialize crank and crank thread create".into(),
    )?;
    Ok(())
}

pub fn crank_delete(client: &Client, market_keys: &MarketKeys, id: String) -> ClientResult<()> {
    let crank_pubkey =
        openbook_crank::state::Crank::pubkey(client.payer_pubkey(), market_keys.market);
    let crank_thread_pubkey =
        clockwork_client::thread::state::Thread::pubkey(client.payer_pubkey(), id.clone());

    let thread_delete_ix = thread_delete(
        client.payer_pubkey(),
        client.payer_pubkey(),
        crank_thread_pubkey,
    );

    let crank_delete_ix = Instruction {
        program_id: openbook_crank::ID,
        accounts: vec![
            AccountMeta::new_readonly(client.payer_pubkey(), true),
            AccountMeta::new(client.payer_pubkey(), false),
            AccountMeta::new(crank_pubkey, false),
        ],
        data: openbook_crank::instruction::Delete {}.data(),
    };

    sign_send_and_confirm_tx(
        client,
        vec![crank_delete_ix, thread_delete_ix],
        None,
        "crank delete".into(),
    )?;

    Ok(())
}
