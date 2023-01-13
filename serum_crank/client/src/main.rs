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

// #[cfg(not(feature = "mainnet"))]
// fn main2() -> ClientResult<()> {
//     // Creating a Client with your default paper keypair as payer
//     let client = default_client();
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     // Security:
//     // Note that we are using your default Solana paper keypair as the thread authority.
//     // Feel free to use whichever authority is appropriate for your use case.
//     let thread_authority = client.payer_pubkey();
//     let bob = Keypair::new();
//     // airdrop a bunch bc it's expensive to setup a dex market and for all of the txs lol
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     client.airdrop(&bob.pubkey(), 2 * LAMPORTS_PER_SOL)?;
//     // setup market
//     let market_keys = setup_market(&client)?;
//     print_market_keys(&market_keys)?;
//     // derive serum_crank PDAs
//     let crank = serum_crank::state::Crank::pubkey(market_keys.market);
//     let crank_thread = clockwork_sdk::state::Thread::pubkey(thread_authority, "crank".into());
//     print_explorer_link(crank, "crank".into())?;
//     print_explorer_link(crank_thread, "crank_thread".into())?;
//     // init serum_crank program
//     initialize_serum_crank(&client, crank, crank_thread, &market_keys)?;
//     // Create wallets for alice and bob
//     let alice_mint_a_wallet = client.create_token_account_with_lamports(
//         &client.payer_pubkey(),
//         &market_keys.pc_mint,
//         LAMPORTS_PER_SOL,
//     )?;
//     let bob_mint_b_wallet = client.create_token_account_with_lamports(
//         &bob.pubkey(),
//         &market_keys.coin_mint,
//         LAMPORTS_PER_SOL,
//     )?;
//     // mint to alice and bob's wallets
//     client.mint_to(
//         client.payer(),
//         &market_keys.pc_mint,
//         &alice_mint_a_wallet.pubkey(),
//         1_000_000_000_000_000,
//         9,
//     )?;
//     client.mint_to(
//         client.payer(),
//         &market_keys.coin_mint,
//         &bob_mint_b_wallet.pubkey(),
//         1_000_000_000_000_000,
//         9,
//     )?;
//     // initialize open order accounts for alice and bob
//     let mut oo_account_alice = None;
//     init_open_orders_account(
//         &client,
//         &openbook_dex_pk(),
//         client.payer(),
//         &market_keys,
//         &mut oo_account_alice,
//     )?;
//     let mut oo_account_bob = None;
//     init_open_orders_account(
//         &client,
//         &openbook_dex_pk(),
//         &bob,
//         &market_keys,
//         &mut oo_account_bob,
//     )?;
//     // place orders
//     for _ in 0..5 {
//         place_order(
//             &client,
//             &openbook_dex_pk(),
//             &bob,
//             &bob_mint_b_wallet.pubkey(),
//             &market_keys,
//             &mut oo_account_bob,
//             NewOrderInstructionV3 {
//                 side: Side::Ask,
//                 limit_price: NonZeroU64::new(500).unwrap(),
//                 max_coin_qty: NonZeroU64::new(1_000).unwrap(),
//                 max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
//                 order_type: OrderType::Limit,
//                 client_order_id: 019269,
//                 self_trade_behavior: SelfTradeBehavior::DecrementTake,
//                 limit: std::u16::MAX,
//             },
//         )?;
//         place_order(
//             &client,
//             &openbook_dex_pk(),
//             client.payer(),
//             &alice_mint_a_wallet.pubkey(),
//             &market_keys,
//             &mut oo_account_alice,
//             NewOrderInstructionV3 {
//                 side: Side::Bid,
//                 limit_price: NonZeroU64::new(500).unwrap(),
//                 max_coin_qty: NonZeroU64::new(1_000).unwrap(),
//                 max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
//                 order_type: OrderType::Limit,
//                 client_order_id: 019269,
//                 self_trade_behavior: SelfTradeBehavior::DecrementTake,
//                 limit: std::u16::MAX,
//             },
//         )?;
//     }
//     Ok(())
// }

fn main() -> ClientResult<()> {
    let client = default_client();

    // SOL/USDC
    // let sol_usdc_market_keys = MarketKeys {
    //     market: Pubkey::from_str("8BnEgHoWFysVcuFFX7QztDmzuH8r5ZFvyP3sYwn1XTh6").unwrap(),
    //     event_q: Pubkey::from_str("8CvwxZ9Db6XbLD46NZwwmVDZZRDy7eydFcAGkXKh9axa").unwrap(),
    //     bids: Pubkey::from_str("5jWUncPNBMZJ3sTHKmMLszypVkoRK6bfEQMQUHweeQnh").unwrap(),
    //     asks: Pubkey::from_str("EaXdHx7x3mdGA38j5RSmKYSXMzAFzzUXCLNBEDXDn1d5").unwrap(),
    //     coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
    //     coin_vault: Pubkey::from_str("6A5NHCj1yF6urc9wZNe6Bcjj4LVszQNj5DwAWG97yzMu").unwrap(),
    //     coin_wallet: Pubkey::from_str("FZnhkDzQeNPZb4VADuucxymVARRRWKxNDh4FsNSSSwAP").unwrap(),
    //     pc_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
    //     pc_vault: Pubkey::from_str("CKxTHwM9fPMRRvZmFnFoqKNd9pQR21c5Aq9bh5h9oghX").unwrap(),
    //     pc_wallet: Pubkey::from_str("9d7WcMvuk9pU5EnNbUDJzuNdsQjaiJo5G7rLFtAozp17").unwrap(),
    //     vault_signer: Pubkey::from_str("CTz5UMLQm2SRWHzQnU62Pi4yJqbNGjgRBHqqp6oDHfF7").unwrap(),
    // };

    let doggo_doge_market_keys = MarketKeys {
        market: Pubkey::from_str("9fD2u4PbBoN8y3vvAtLMpVDFw2ThPWA11PV6CcsiSnu5").unwrap(),
        event_q: Pubkey::from_str("Fa4mRrRPTEbkW8hs1ER3EKdtxqxGiD63JuL7Dk2Ew7g8").unwrap(),
        bids: Pubkey::from_str("AwEDEUgZP9nN8nvS7zEfWggLFf2k8obc6jsem6Jedfmh").unwrap(),
        asks: Pubkey::from_str("4ABNf3dCWQg6NN5Xc1NXWN7m5vm1zutFe5RSXhSKn5KR").unwrap(),
        coin_mint: Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(),
        coin_vault: Pubkey::from_str("C6fNmJmvbjiU24Nc6GtFA85s6YhRjzTdz9o671EVKkqF").unwrap(),
        coin_wallet: Pubkey::from_str("27VkqLYvBKfnosJcvfPbwqomVsVZA81LF5pYb4C36MLU").unwrap(),
        pc_mint: Pubkey::from_str("Doggoyb1uHFJGFdHhJf8FKEBUMv58qo98CisWgeD7Ftk").unwrap(),
        pc_vault: Pubkey::from_str("2P6WAjDg6ibRjNDYatmLow9feCW85tJcgZyaZnGi8Rxo").unwrap(),
        pc_wallet: Pubkey::from_str("BJKQiDRtVU5ecJ7UzNSmBCUmNFWqSoyK5vSS4RynnDhf").unwrap(),
        vault_signer: Pubkey::from_str("DnVM9L2RKH9xMnbHbNt239PVxcFFWTsTb2aKu9aATqZc").unwrap(),
    };

    initialize_openbook_crank(
        &client,
        &doggo_doge_market_keys,
        "DOGGO_USDC_OPENBOOK_TEST_1".into(),
    )?;

    Ok(())
}

fn initialize_openbook_crank(
    client: &Client,
    market_keys: &MarketKeys,
    id: String,
) -> ClientResult<()> {
    let crank_pubkey =
        serum_crank::state::Crank::pubkey(client.payer_pubkey(), market_keys.market, id.clone());
    let crank_thread_pubkey =
        clockwork_client::thread::state::Thread::pubkey(client.payer_pubkey(), id.clone());

    print_explorer_link(crank_thread_pubkey, "crank_thread".into(), Cluster::Mainnet)?;

    // define initialize ix
    let initialize_ix = Instruction {
        program_id: serum_crank::ID,
        accounts: vec![
            AccountMeta::new(crank_pubkey, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new_readonly(market_keys.event_q, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new_readonly(market_keys.pc_mint, false),
            AccountMeta::new_readonly(market_keys.pc_vault, false),
            AccountMeta::new_readonly(market_keys.pc_wallet, false),
            AccountMeta::new_readonly(market_keys.coin_mint, false),
            AccountMeta::new_readonly(market_keys.coin_vault, false),
            AccountMeta::new_readonly(market_keys.coin_wallet, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(market_keys.vault_signer, false),
        ],
        data: serum_crank::instruction::Initialize { id: id.clone() }.data(),
    };

    // create thread with read events ix
    let thread_create = thread_create(
        client.payer_pubkey(),
        id,
        Instruction {
            program_id: serum_crank::ID,
            accounts: vec![
                AccountMeta::new(crank_pubkey.key(), false),
                AccountMeta::new(crank_thread_pubkey.key(), true),
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new_readonly(market_keys.event_q, false),
                AccountMeta::new_readonly(market_keys.market, false),
                AccountMeta::new(PAYER_PUBKEY, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: serum_crank::instruction::ReadEvents.data(),
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

// fn initialize_serum_crank(
//     client: &Client,
//     crank: Pubkey,
//     crank_thread: Pubkey,
//     market_keys: &MarketKeys,
// ) -> ClientResult<()> {
//     client.airdrop(&crank_thread, LAMPORTS_PER_SOL)?;
//     // destructor struct for convenience
//     let MarketKeys {
//         event_q,
//         market,
//         pc_mint,
//         pc_vault,
//         coin_mint,
//         coin_vault,
//         ..
//     } = *market_keys;
//     // define initialize ix
//     let initialize_ix = Instruction {
//         program_id: serum_crank::ID,
//         accounts: vec![
//             AccountMeta::new(crank, false),
//             AccountMeta::new_readonly(openbook_dex_pk(), false),
//             AccountMeta::new_readonly(event_q, false),
//             AccountMeta::new_readonly(market, false),
//             AccountMeta::new_readonly(pc_mint, false),
//             AccountMeta::new_readonly(pc_vault, false),
//             AccountMeta::new_readonly(coin_mint, false),
//             AccountMeta::new_readonly(coin_vault, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new_readonly(system_program::ID, false),
//         ],
//         data: serum_crank::instruction::Initialize.data(),
//     };
//     // create thread with read events ix
//     let thread_create = thread_create(
//         client.payer_pubkey(),
//         "crank".into(),
//         Instruction {
//             program_id: serum_crank::ID,
//             accounts: vec![
//                 AccountMeta::new(crank.key(), false),
//                 AccountMeta::new(crank_thread.key(), true),
//                 AccountMeta::new_readonly(openbook_dex_pk(), false),
//                 AccountMeta::new_readonly(event_q, false),
//                 AccountMeta::new_readonly(market, false),
//                 AccountMeta::new_readonly(pc_vault, false),
//                 AccountMeta::new_readonly(coin_vault, false),
//                 AccountMeta::new(PAYER_PUBKEY, true),
//                 AccountMeta::new_readonly(system_program::ID, false),
//             ],
//             data: serum_crank::instruction::ReadEvents.data(),
//         }
//         .into(),
//         client.payer_pubkey(),
//         crank_thread,
//         Trigger::Account {
//             address: event_q,
//             offset: 8 + 8,
//             size: 8,
//         },
//     );
//     sign_send_and_confirm_tx(
//         client,
//         vec![initialize_ix, thread_create],
//         None,
//         "initialize crank and thread_create".into(),
//     )?;
//     Ok(())
// }

// pub fn init_open_orders_account(
//     client: &Client,
//     program_id: &Pubkey,
//     owner: &Keypair,
//     market_keys: &MarketKeys,
//     orders: &mut Option<Pubkey>,
// ) -> ClientResult<()> {
//     let orders_keypair;
//     let mut ix = Vec::new();
//     let mut signers = Vec::new();
//     let orders_pubkey = match *orders {
//         Some(pk) => pk,
//         None => {
//             let (orders_key, instruction) = create_dex_account(
//                 client,
//                 program_id,
//                 &client.payer_pubkey(),
//                 size_of::<OpenOrders>(),
//             )
//             .unwrap();
//             orders_keypair = orders_key;
//             signers.push(&orders_keypair);
//             ix.push(instruction);
//             orders_keypair.pubkey()
//         }
//     };
//     *orders = Some(orders_pubkey);
//     ix.push(
//         init_open_orders_ix(
//             program_id,
//             &orders_pubkey,
//             &owner.pubkey(),
//             &market_keys.market,
//         )
//         .unwrap(),
//     );
//     signers.push(owner);
//     signers.push(client.payer());
//     sign_send_and_confirm_tx(
//         client,
//         ix,
//         Some(signers),
//         "create open orders account".into(),
//     )?;
//     Ok(())
// }

// pub fn place_order(
//     client: &Client,
//     program_id: &Pubkey,
//     payer: &Keypair,
//     wallet: &Pubkey,
//     market_keys: &MarketKeys,
//     orders: &mut Option<Pubkey>,
//     new_order: anchor_spl::dex::serum_dex::instruction::NewOrderInstructionV3,
// ) -> ClientResult<()> {
//     let mut instructions = Vec::new();
//     let orders_keypair;
//     let mut signers = Vec::new();
//     let orders_pubkey = match *orders {
//         Some(pk) => pk,
//         None => {
//             let (orders_key, instruction) =
//                 create_dex_account(client, program_id, &payer.pubkey(), size_of::<OpenOrders>())?;
//             orders_keypair = orders_key;
//             signers.push(&orders_keypair);
//             instructions.push(instruction);
//             orders_keypair.pubkey()
//         }
//     };
//     *orders = Some(orders_pubkey);
//     let _side = new_order.side;
//     let data =
//         anchor_spl::dex::serum_dex::instruction::MarketInstruction::NewOrderV3(new_order).pack();
//     let instruction = Instruction {
//         program_id: *program_id,
//         data,
//         accounts: vec![
//             AccountMeta::new(market_keys.market, false),
//             AccountMeta::new(orders_pubkey, false),
//             AccountMeta::new(market_keys.req_q, false),
//             AccountMeta::new(market_keys.event_q, false),
//             AccountMeta::new(market_keys.bids, false),
//             AccountMeta::new(market_keys.asks, false),
//             AccountMeta::new(*wallet, false),
//             AccountMeta::new_readonly(payer.pubkey(), true),
//             AccountMeta::new(market_keys.coin_vault, false),
//             AccountMeta::new(market_keys.pc_vault, false),
//             AccountMeta::new_readonly(token::spl_token::ID, false),
//             AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
//         ],
//     };
//     instructions.push(instruction);
//     signers.push(payer);
//     signers.push(client.payer());
//     sign_send_and_confirm_tx(client, instructions, Some(signers), "place_order".into())?;
//     Ok(())
// }

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
