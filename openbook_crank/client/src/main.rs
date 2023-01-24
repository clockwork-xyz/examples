mod utils;

use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_client::{
        thread::{instruction::thread_create, state::Trigger},
        Client, ClientResult,
    },
    clockwork_sdk::utils::PAYER_PUBKEY,
    solana_sdk::{instruction::Instruction, signer::keypair::read_keypair_file},
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
        coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
        coin_vault: Pubkey::from_str("6A5NHCj1yF6urc9wZNe6Bcjj4LVszQNj5DwAWG97yzMu").unwrap(),
        coin_wallet: Pubkey::from_str("FZnhkDzQeNPZb4VADuucxymVARRRWKxNDh4FsNSSSwAP").unwrap(),
        pc_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
        pc_vault: Pubkey::from_str("CKxTHwM9fPMRRvZmFnFoqKNd9pQR21c5Aq9bh5h9oghX").unwrap(),
        pc_wallet: Pubkey::from_str("9d7WcMvuk9pU5EnNbUDJzuNdsQjaiJo5G7rLFtAozp17").unwrap(),
        vault_signer: Pubkey::from_str("CTz5UMLQm2SRWHzQnU62Pi4yJqbNGjgRBHqqp6oDHfF7").unwrap(),
    };

    // let doggo_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("9fD2u4PbBoN8y3vvAtLMpVDFw2ThPWA11PV6CcsiSnu5").unwrap(),
    //     event_q: Pubkey::from_str("Fa4mRrRPTEbkW8hs1ER3EKdtxqxGiD63JuL7Dk2Ew7g8").unwrap(),
    //     bids: Pubkey::from_str("AwEDEUgZP9nN8nvS7zEfWggLFf2k8obc6jsem6Jedfmh").unwrap(),
    //     asks: Pubkey::from_str("4ABNf3dCWQg6NN5Xc1NXWN7m5vm1zutFe5RSXhSKn5KR").unwrap(),
    //     coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     coin_vault: Pubkey::from_str("C6fNmJmvbjiU24Nc6GtFA85s6YhRjzTdz9o671EVKkqF").unwrap(),
    //     coin_wallet: Pubkey::from_str("27VkqLYvBKfnosJcvfPbwqomVsVZA81LF5pYb4C36MLU").unwrap(),
    //     pc_mint: Pubkey::from_str("Doggoyb1uHFJGFdHhJf8FKEBUMv58qo98CisWgeD7Ftk").unwrap(),
    //     pc_vault: Pubkey::from_str("2P6WAjDg6ibRjNDYatmLow9feCW85tJcgZyaZnGi8Rxo").unwrap(),
    //     pc_wallet: Pubkey::from_str("BJKQiDRtVU5ecJ7UzNSmBCUmNFWqSoyK5vSS4RynnDhf").unwrap(),
    //     vault_signer: Pubkey::from_str("DnVM9L2RKH9xMnbHbNt239PVxcFFWTsTb2aKu9aATqZc").unwrap(),
    // };

    // let basis_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("FfP1cFGHeUfJmJKWhEvA8eUArCQvVgVHodt2AfLdWMdf").unwrap(),
    //     event_q: Pubkey::from_str("DPabHqDzAWN4fEzxY4Wfa4aLuRLmGyUAFAzCcAnauUiG").unwrap(),
    //     bids: Pubkey::from_str("2ZcywQ1xA37hLYpaDW9a6J6LxyTxjvJf7b7nBfxutbL2").unwrap(),
    //     asks: Pubkey::from_str("CQ5MQBXS6gfcdkJNQujU4viqrdpvN85A2Fa8qERCMSrY").unwrap(),
    //     coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     coin_vault: Pubkey::from_str("GpFgz9Pjf6MqW5CNxqWzmSxhpXpUyL32khUbLpQ3X968").unwrap(),
    //     coin_wallet: Pubkey::from_str("5AZdzEh5qvexHgJYCKXZD6LRDLMj2L7EWQETM3nyu9UE").unwrap(),
    //     pc_mint: Pubkey::from_str("Basis9oJw9j8cw53oMV7iqsgo6ihi9ALw4QR31rcjUJa").unwrap(),
    //     pc_vault: Pubkey::from_str("jrLgmkQ3ZyGvv2rxaMvg7i7QVwzTD5Co8LVWE1gSqrs").unwrap(),
    //     pc_wallet: Pubkey::from_str("8tZ33NZ6geKLrcN6yVrdLGbsY3bmjJ51kKCNtJaU7xE1").unwrap(),
    //     vault_signer: Pubkey::from_str("JCrdR5BFUZCd4ZBWTNVFP95PsHGn4QHyn5sw3s4QnJ3X").unwrap(),
    // };

    initialize_openbook_crank(
        &client,
        &sol_usdc_market_keys,
        "SOL_USDC_OPENBOOK_TEST_2".into(),
    )?;

    Ok(())
}

fn initialize_openbook_crank(
    client: &Client,
    market_keys: &MarketKeys,
    id: String,
) -> ClientResult<()> {
    let crank_pubkey =
        openbook_crank::state::Crank::pubkey(client.payer_pubkey(), market_keys.market, id.clone());
    let crank_thread_pubkey =
        clockwork_client::thread::state::Thread::pubkey(client.payer_pubkey(), id.clone());

    print_explorer_link(crank_thread_pubkey, "crank_thread".into(), Cluster::Mainnet)?;

    // define initialize ix
    let initialize_ix = Instruction {
        program_id: openbook_crank::ID,
        accounts: vec![
            AccountMeta::new(crank_pubkey, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new_readonly(market_keys.event_q, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new_readonly(market_keys.pc_mint, false),
            AccountMeta::new_readonly(market_keys.pc_vault, false),
            AccountMeta::new_readonly(market_keys.coin_mint, false),
            AccountMeta::new_readonly(market_keys.coin_vault, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: openbook_crank::instruction::Initialize { id: id.clone() }.data(),
    };

    // create thread with read events ix
    let thread_create = thread_create(
        client.payer_pubkey(),
        id,
        vec![Instruction {
            program_id: openbook_crank::ID,
            accounts: vec![
                AccountMeta::new(crank_pubkey.key(), false),
                AccountMeta::new(crank_thread_pubkey.key(), true),
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new_readonly(market_keys.event_q, false),
                AccountMeta::new_readonly(market_keys.market, false),
                AccountMeta::new(PAYER_PUBKEY, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: openbook_crank::instruction::ReadEvents.data(),
        }
        .into()],
        client.payer_pubkey(),
        crank_thread_pubkey,
        Trigger::Account {
            address: market_keys.event_q,
            offset: 8 + 8,
            size: 8,
        },
    );

    sign_send_and_confirm_tx(
        client,
        vec![initialize_ix, thread_create],
        None,
        "initialize crank and thread_create".into(),
    )?;
    Ok(())
}

pub fn print_explorer_link(address: Pubkey, label: String, cluster: Cluster) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster={}",
        label.to_string(),
        address,
        cluster.value()
    );

    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum Cluster {
    Localnet,
    Devnet,
    Mainnet,
}

impl Cluster {
    fn value(&self) -> &str {
        match *self {
            Cluster::Localnet => "custom",
            Cluster::Devnet => "devnet",
            Cluster::Mainnet => "null",
        }
    }
}

fn default_client() -> Client {
    #[cfg(not(feature = "localnet"))]
    let host = "https://api.mainnet-beta.solana.com";
    #[cfg(feature = "localnet")]
    let host = "http://localhost:8899";

    let config_file = solana_cli_config::CONFIG_FILE.as_ref().unwrap().as_str();
    let config = solana_cli_config::Config::load(config_file).unwrap();
    let payer = read_keypair_file(&config.keypair_path).unwrap();
    Client::new(payer, host.into())
}
