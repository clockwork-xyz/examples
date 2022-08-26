use std::num::NonZeroU64;

use anchor_spl::dex::serum_dex::{
    instruction::{NewOrderInstructionV3, SelfTradeBehavior},
    matching::{OrderType, Side},
};

mod utils;

use {
    anchor_lang::{prelude::*, solana_program::sysvar, system_program, InstructionData},
    anchor_spl::{
        dex::serum_dex::{instruction::initialize_market, state::OpenOrders},
        token,
    },
    serum_common::client::rpc::mint_to_new_account,
    solana_client_helpers::{Client, ClientResult, RpcClient, SplToken},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer,
    },
    std::mem::size_of,
    utils::*,
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("http://localhost:8899");
    // let client = RpcClient::new("https://api.devnet.solana.com");
    let payer = Keypair::new();
    let bob = Keypair::new();
    let client = Client { client, payer };

    // airdrop a bunch bc it's expensive to setup a dex market and for all of the txs lol
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&bob.pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // setup market
    let market_keys = setup_market(&client)?;

    // Derive PDAsCx#5K4wY8yn
    // let crank = serum_crank::state::Crank::pubkey();
    // let crank_queue = clockwork_crank::state::Queue::pubkey(crank, "crank".to_string());

    print_explorer_link(market_keys.market, "market".to_string())?;
    print_explorer_link(market_keys.event_q, "event_queue".to_string())?;
    // print_explorer_link(crank_queue, "queue".to_string())?;

    // initialize_serum_crank(&client, crank, crank_queue, &market_keys)?;

    // // create second party wallets to then mint to
    // let coin_wallet_key_2 = mint_to_new_account(
    //     &client,
    //     &bob,
    //     &client.payer(),
    //     &market_keys.coin_mint,
    //     1_000_000_000_000_000,
    // )
    // .unwrap();

    // let pc_wallet_key_2 = mint_to_new_account(
    //     &client,
    //     &client.payer(),
    //     &client.payer(),
    //     &market_keys.pc_mint,
    //     1_000_000_000_000_000,
    // )
    // .unwrap();

    // println!("serum explorer: https://serum-explorer.vercel.app/market/{}?network=custom&customRPC=http%3A%2F%2Flocalhost%3A8899", market_keys.market);
    // println!(
    //     " coin_vault: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.coin_vault
    // );
    // println!(
    //     "coin_wallet: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.pc_wallet_key.pubkey()
    // );
    // println!(
    //     "  coin_mint: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.pc_mint
    // );
    // println!(
    //     "   pc_vault: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.pc_vault
    // );
    // println!(
    //     "  pc_wallet: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.pc_wallet_key.pubkey()
    // );
    // println!(
    //     "    pc_mint: https://explorer.solana.com/address/{}?cluster=custom",
    //     market_keys.pc_mint
    // );

    // let mut oo_account_bob = None;

    // init_open_orders_account(
    //     &client,
    //     &anchor_spl::dex::ID,
    //     &bob,
    //     &market_keys,
    //     &mut oo_account_bob,
    // )?;

    // let mut oo_account_alice = None;

    // init_open_orders_account(
    //     &client,
    //     &anchor_spl::dex::ID,
    //     &client.payer(),
    //     &market_keys,
    //     &mut oo_account_alice,
    // )?;

    // open orders account
    // let mut orders = None;

    // Derive ATA pubkeys
    // let payer_mint_a_token_account = anchor_spl::associated_token::get_associated_token_address(
    //     &client.payer_pubkey(),
    //     &market_keys.pc_mint,
    // );
    // let payer_mint_b_token_account = anchor_spl::associated_token::get_associated_token_address(
    //     &client.payer_pubkey(),
    //     &market_keys.coin_mint,
    // );
    // let investment_mint_a_token_account =
    //     anchor_spl::associated_token::get_associated_token_address(
    //         &investment,
    //         &market_keys.pc_mint,
    //     );
    // let investment_mint_b_token_account =
    //     anchor_spl::associated_token::get_associated_token_address(
    //         &investment,
    //         &market_keys.coin_mint,
    //     );

    // for _ in 0..2 {
    //     place_order(
    //         &client,
    //         &anchor_spl::dex::ID,
    //         &bob,
    //         &coin_wallet_key_2.pubkey(),
    //         &market_keys,
    //         &mut oo_account_bob,
    //         NewOrderInstructionV3 {
    //             side: Side::Ask,
    //             limit_price: NonZeroU64::new(500).unwrap(),
    //             max_coin_qty: NonZeroU64::new(1_000).unwrap(),
    //             max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
    //             order_type: OrderType::Limit,
    //             client_order_id: 019269,
    //             self_trade_behavior: SelfTradeBehavior::DecrementTake,
    //             limit: std::u16::MAX,
    //         },
    //     )?;
    // }

    // for _ in 0..2 {
    //     place_order(
    //         &client,
    //         &anchor_spl::dex::ID,
    //         &client.payer(),
    //         &pc_wallet_key_2.pubkey(),
    //         &market_keys,
    //         &mut oo_account_alice,
    //         NewOrderInstructionV3 {
    //             side: Side::Bid,
    //             limit_price: NonZeroU64::new(500).unwrap(),
    //             max_coin_qty: NonZeroU64::new(1_000).unwrap(),
    //             max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
    //             order_type: OrderType::Limit,
    //             client_order_id: 019269,
    //             self_trade_behavior: SelfTradeBehavior::DecrementTake,
    //             limit: std::u16::MAX,
    //         },
    //     )?;
    // }

    // create_investment_and_deposit(
    //     &client,
    //     investment,
    //     investment_mint_a_token_account,
    //     investment_mint_b_token_account,
    //     &market_keys,
    //     &mut orders,
    //     payer_mint_a_token_account,
    //     payer_mint_b_token_account,
    //     queue,
    //     task,
    // )?;

    Ok(())
}

fn setup_market(client: &Client) -> ClientResult<MarketKeys> {
    // generate 2 mints to list on market
    let coin_mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    let pc_mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    // get market listing keys
    let (listing_keys, mut ix) = gen_listing_params(
        client,
        &anchor_spl::dex::ID,
        &client.payer_pubkey(),
        &coin_mint,
        &pc_mint,
    )?;

    // destructuring market listing keys
    let ListingKeys {
        market_key,
        req_q_key,
        event_q_key,
        bids_key,
        asks_key,
        vault_signer,
        vault_signer_nonce,
    } = listing_keys;

    // create ata vaults for the respective mints
    let coin_vault =
        client.create_associated_token_account(&client.payer(), &vault_signer, &coin_mint)?;

    let pc_vault =
        client.create_associated_token_account(&client.payer(), &vault_signer, &pc_mint)?;

    // get the init market ix
    let init_market_ix = initialize_market(
        &market_key.pubkey(),
        &anchor_spl::dex::ID,
        &coin_mint,
        &pc_mint,
        &coin_vault,
        &pc_vault,
        None,
        None,
        &bids_key.pubkey(),
        &asks_key.pubkey(),
        &req_q_key.pubkey(),
        &event_q_key.pubkey(),
        1_000_000_000,
        1_000_000_000,
        vault_signer_nonce,
        100,
    )
    .unwrap();

    // add init_market_ix to vector
    ix.push(init_market_ix);

    sign_send_and_confirm_tx(
        &client,
        ix,
        Some(vec![
            client.payer(),
            &market_key,
            &req_q_key,
            &event_q_key,
            &bids_key,
            &asks_key,
            &req_q_key,
            &event_q_key,
        ]),
        "setup_market".to_string(),
    )?;

    // create wallets to then mint to
    let coin_wallet_key = mint_to_new_account(
        &client,
        &client.payer(),
        &client.payer(),
        &coin_mint,
        1_000_000_000_000_000,
    )
    .unwrap();

    let pc_wallet_key = mint_to_new_account(
        &client,
        &client.payer(),
        &client.payer(),
        &pc_mint,
        1_000_000_000_000_000,
    )
    .unwrap();

    Ok(MarketKeys {
        market: market_key.pubkey(),
        req_q: req_q_key.pubkey(),
        event_q: event_q_key.pubkey(),
        bids: bids_key.pubkey(),
        asks: asks_key.pubkey(),
        coin_mint,
        coin_vault,
        pc_mint,
        pc_vault,
        vault_signer,
        pc_wallet_key,
        coin_wallet_key,
    })
}

// fn initialize_serum_crank(
//     client: &Client,
//     crank: Pubkey,
//     crank_queue: Pubkey,
//     market_keys: &MarketKeys,
// ) -> ClientResult<()> {
//     let initialize_ix = Instruction {
//         program_id: serum_crank::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(clockwork_crank::ID, false),
//             AccountMeta::new(crank, false),
//             AccountMeta::new(crank_queue, false),
//             AccountMeta::new_readonly(anchor_spl::dex::ID, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new_readonly(system_program::ID, false),
//             // Extra Accounts
//             AccountMeta::new(market_keys.market, false),
//             AccountMeta::new(market_keys.pc_vault, false),
//             AccountMeta::new(market_keys.coin_vault, false),
//             AccountMeta::new(market_keys.event_q, false),
//         ],
//         data: serum_crank::instruction::Initialize.data(),
//     };

//     sign_send_and_confirm_tx(
//         &client,
//         [initialize_ix].to_vec(),
//         None,
//         "initialize_serum_crank".to_string(),
//     )?;

//     client.airdrop(&crank_queue, LAMPORTS_PER_SOL)?;

//     Ok(())
// }

// fn _create_investment_and_deposit(
//     client: &Client,
//     investment: Pubkey,
//     investment_mint_a_token_account: Pubkey,
//     investment_mint_b_token_account: Pubkey,
//     market_keys: &MarketKeys,
//     orders: &mut Option<Pubkey>,
//     payer_mint_a_token_account: Pubkey,
//     payer_mint_b_token_account: Pubkey,
//     queue: Pubkey,
//     task: Pubkey,
// ) -> ClientResult<()> {
//     init_dex_account(client, orders)?;

//     let create_investment_ix = Instruction {
//         program_id: investments_program::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(associated_token::ID, false),
//             AccountMeta::new_readonly(anchor_spl::dex::ID, false),
//             AccountMeta::new(investment, false),
//             AccountMeta::new(investment_mint_a_token_account, false),
//             AccountMeta::new(investment_mint_b_token_account, false),
//             AccountMeta::new_readonly(market_keys.pc_mint, false),
//             AccountMeta::new_readonly(market_keys.coin_mint, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new(payer_mint_a_token_account, false),
//             AccountMeta::new(payer_mint_b_token_account, false),
//             AccountMeta::new(queue, false),
//             AccountMeta::new_readonly(sysvar::rent::ID, false),
//             AccountMeta::new_readonly(clockwork_scheduler::ID, false),
//             AccountMeta::new_readonly(system_program::ID, false),
//             AccountMeta::new(task, false),
//             AccountMeta::new_readonly(token::ID, false),
//             // Extra accounts
//             AccountMeta::new(market_keys.market, false),
//             AccountMeta::new(market_keys.coin_vault, false),
//             AccountMeta::new(market_keys.pc_vault, false),
//             AccountMeta::new(market_keys.req_q, false),
//             AccountMeta::new(market_keys.event_q, false),
//             AccountMeta::new(market_keys.bids, false),
//             AccountMeta::new(market_keys.asks, false),
//             AccountMeta::new(orders.unwrap(), false),
//             AccountMeta::new(market_keys.vault_signer, false),
//             AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
//             AccountMeta::new(market_keys.coin_wallet_key.pubkey(), false),
//         ],
//         data: investments_program::instruction::CreateInvestment {
//             swap_amount: 10_000_000,
//         }
//         .data(),
//     };

//     let create_orders_ix = Instruction {
//         program_id: investments_program::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(anchor_spl::dex::ID, false),
//             AccountMeta::new_readonly(investment, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new_readonly(sysvar::rent::ID, false),
//             AccountMeta::new_readonly(system_program::ID, false),
//             // Extra accounts
//             AccountMeta::new_readonly(market_keys.market, false),
//             AccountMeta::new(orders.unwrap(), false),
//         ],
//         data: investments_program::instruction::CreateOrders {}.data(),
//     };

//     sign_send_and_confirm_tx(
//         &client,
//         [create_investment_ix, create_orders_ix].to_vec(),
//         None,
//         "create_investment_and_orders".to_string(),
//     )?;

//     println!("payer: {}", client.payer_pubkey());
//     println!("investment: {}", investment);
//     println!("mint_a: {}", market_keys.pc_mint);
//     println!(
//         "investment_mint_a_token_account: {}",
//         investment_mint_a_token_account
//     );
//     println!(
//         "investment_mint_b_token_account: {}",
//         investment_mint_b_token_account
//     );
//     println!("payer_mint_a_token_account: {}", payer_mint_a_token_account);
//     println!("payer_mint_b_token_account: {}", payer_mint_b_token_account);
//     println!("queue: {}", queue);
//     println!("mint_a_vault: {}", market_keys.pc_vault);
//     println!("mint_b_vault: {}", market_keys.coin_vault);

//     // mint to payer's mint A ATA
//     client.mint_to(
//         &client.payer(),
//         &market_keys.pc_mint,
//         &payer_mint_a_token_account,
//         2 * LAMPORTS_PER_SOL,
//         9,
//     )?;

//     let deposit_ix = Instruction {
//         program_id: investments_program::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(associated_token::ID, false),
//             AccountMeta::new_readonly(investment, false),
//             AccountMeta::new(investment_mint_a_token_account, false),
//             AccountMeta::new_readonly(market_keys.pc_mint, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new(payer_mint_a_token_account, false),
//             AccountMeta::new_readonly(sysvar::rent::ID, false),
//             AccountMeta::new_readonly(system_program::ID, false),
//             AccountMeta::new_readonly(token::ID, false),
//         ],
//         data: investments_program::instruction::Deposit {
//             amount: 2 * LAMPORTS_PER_SOL,
//         }
//         .data(),
//     };

//     sign_send_and_confirm_tx(
//         &client,
//         [deposit_ix].to_vec(),
//         None,
//         "deposit_ix".to_string(),
//     )?;

//     Ok(())
// }

fn _init_dex_account(client: &Client, orders: &mut Option<Pubkey>) -> ClientResult<()> {
    let orders_keypair;
    let mut ix = Vec::new();
    let mut signers = Vec::new();

    let orders_pk = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) = create_dex_account(
                client,
                &anchor_spl::dex::ID,
                &client.payer_pubkey(),
                size_of::<OpenOrders>(),
            )?;
            orders_keypair = orders_key;
            signers.push(&orders_keypair);
            ix.push(instruction);
            orders_keypair.pubkey()
        }
    };

    *orders = Some(orders_pk);

    signers.push(client.payer());

    sign_send_and_confirm_tx(client, ix, Some(signers), "init_dex_account".to_string())?;

    Ok(())
}

pub fn init_open_orders_account(
    client: &Client,
    program_id: &Pubkey,
    owner: &Keypair,
    market_keys: &MarketKeys,
    orders: &mut Option<Pubkey>,
) -> ClientResult<()> {
    let orders_keypair;
    let mut ix = Vec::new();
    let mut signers = Vec::new();

    let orders_pubkey = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) = create_dex_account(
                client,
                program_id,
                &client.payer_pubkey(),
                size_of::<OpenOrders>(),
            )
            .unwrap();
            orders_keypair = orders_key;
            signers.push(&orders_keypair);
            ix.push(instruction);
            orders_keypair.pubkey()
        }
    };
    *orders = Some(orders_pubkey);
    ix.push(
        init_open_orders_ix(
            program_id,
            &orders_pubkey,
            &owner.pubkey(),
            &market_keys.market,
        )
        .unwrap(),
    );

    signers.push(owner);
    signers.push(client.payer());

    sign_send_and_confirm_tx(
        client,
        ix,
        Some(signers),
        "create open orders account".into(),
    )?;

    Ok(())
}

pub fn init_open_orders_ix(
    program_id: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    market: &Pubkey,
) -> ClientResult<Instruction> {
    let data = anchor_spl::dex::serum_dex::instruction::MarketInstruction::InitOpenOrders.pack();
    let accounts: Vec<AccountMeta> = vec![
        AccountMeta::new(*open_orders, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(sysvar::rent::ID, false),
    ];
    Ok(Instruction {
        program_id: *program_id,
        data,
        accounts,
    })
}

pub fn place_order(
    client: &Client,
    program_id: &Pubkey,
    payer: &Keypair,
    wallet: &Pubkey,
    market_keys: &MarketKeys,
    orders: &mut Option<Pubkey>,
    new_order: anchor_spl::dex::serum_dex::instruction::NewOrderInstructionV3,
) -> ClientResult<()> {
    let mut instructions = Vec::new();
    let orders_keypair;
    let mut signers = Vec::new();
    let orders_pubkey = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) =
                create_dex_account(client, program_id, &payer.pubkey(), size_of::<OpenOrders>())?;
            orders_keypair = orders_key;
            signers.push(&orders_keypair);
            instructions.push(instruction);
            orders_keypair.pubkey()
        }
    };
    *orders = Some(orders_pubkey);
    let _side = new_order.side;
    let data =
        anchor_spl::dex::serum_dex::instruction::MarketInstruction::NewOrderV3(new_order).pack();
    let instruction = Instruction {
        program_id: *program_id,
        data,
        accounts: vec![
            AccountMeta::new(market_keys.market, false),
            AccountMeta::new(orders_pubkey, false),
            AccountMeta::new(market_keys.req_q, false),
            AccountMeta::new(market_keys.event_q, false),
            AccountMeta::new(market_keys.bids, false),
            AccountMeta::new(market_keys.asks, false),
            AccountMeta::new(*wallet, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(market_keys.coin_vault, false),
            AccountMeta::new(market_keys.pc_vault, false),
            AccountMeta::new_readonly(token::spl_token::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
        ],
    };
    instructions.push(instruction);
    signers.push(payer);
    signers.push(client.payer());

    sign_send_and_confirm_tx(client, instructions, Some(signers), "place_order".into())?;
    Ok(())
}
