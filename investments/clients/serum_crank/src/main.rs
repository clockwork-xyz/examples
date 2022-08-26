mod utils;

use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    anchor_spl::{
        dex::serum_dex::{
            state::OpenOrders,
            {
                instruction::{NewOrderInstructionV3, SelfTradeBehavior},
                matching::{OrderType, Side},
            },
        },
        token,
    },
    serum_common::client::rpc::mint_to_new_account,
    solana_client_helpers::{Client, ClientResult, RpcClient},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer,
    },
    std::{mem::size_of, num::NonZeroU64},
    utils::*,
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("http://localhost:8899");
    let payer = Keypair::new();
    let client = Client { client, payer };

    let bob = Keypair::new();

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
    print_market_keys(&market_keys)?;

    // derive serum_crank PDAs
    let crank = serum_crank::state::Crank::pubkey(market_keys.market);
    let crank_queue = clockwork_crank::state::Queue::pubkey(crank, "crank".into());

    print_explorer_link(crank, "crank".into())?;
    print_explorer_link(crank_queue, "crank_queue".into())?;

    // init serum_crank program
    initialize_serum_crank(&client, crank, crank_queue, &market_keys)?;

    // Create wallets for alice and bob
    let alice_mint_a_wallet = mint_to_new_account(
        &client,
        &client.payer(),
        &client.payer(),
        &market_keys.pc_mint,
        1_000_000_000_000_000,
    )
    .unwrap();

    let bob_mint_b_wallet = mint_to_new_account(
        &client,
        &bob,
        &client.payer(),
        &market_keys.coin_mint,
        1_000_000_000_000_000,
    )
    .unwrap();

    // initialize open order accounts for alice and bob
    let mut oo_account_alice = None;

    init_open_orders_account(
        &client,
        &anchor_spl::dex::ID,
        &client.payer(),
        &market_keys,
        &mut oo_account_alice,
    )?;

    let mut oo_account_bob = None;

    init_open_orders_account(
        &client,
        &anchor_spl::dex::ID,
        &bob,
        &market_keys,
        &mut oo_account_bob,
    )?;

    // place orders
    for _ in 0..5 {
        place_order(
            &client,
            &anchor_spl::dex::ID,
            &bob,
            &bob_mint_b_wallet.pubkey(),
            &market_keys,
            &mut oo_account_bob,
            NewOrderInstructionV3 {
                side: Side::Ask,
                limit_price: NonZeroU64::new(500).unwrap(),
                max_coin_qty: NonZeroU64::new(1_000).unwrap(),
                max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
                order_type: OrderType::Limit,
                client_order_id: 019269,
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                limit: std::u16::MAX,
            },
        )?;

        place_order(
            &client,
            &anchor_spl::dex::ID,
            &client.payer(),
            &alice_mint_a_wallet.pubkey(),
            &market_keys,
            &mut oo_account_alice,
            NewOrderInstructionV3 {
                side: Side::Bid,
                limit_price: NonZeroU64::new(500).unwrap(),
                max_coin_qty: NonZeroU64::new(1_000).unwrap(),
                max_native_pc_qty_including_fees: NonZeroU64::new(500_000).unwrap(),
                order_type: OrderType::Limit,
                client_order_id: 019269,
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
                limit: std::u16::MAX,
            },
        )?;
    }

    Ok(())
}

fn initialize_serum_crank(
    client: &Client,
    crank: Pubkey,
    crank_queue: Pubkey,
    market_keys: &MarketKeys,
) -> ClientResult<()> {
    client.airdrop(&crank_queue, LAMPORTS_PER_SOL)?;

    let initialize_ix = Instruction {
        program_id: serum_crank::ID,
        accounts: vec![
            AccountMeta::new_readonly(clockwork_crank::ID, false),
            AccountMeta::new(crank, false),
            AccountMeta::new(crank_queue, false),
            AccountMeta::new_readonly(anchor_spl::dex::ID, false),
            AccountMeta::new_readonly(market_keys.event_q, false),
            AccountMeta::new_readonly(market_keys.market, false),
            AccountMeta::new_readonly(market_keys.pc_mint, false),
            AccountMeta::new_readonly(market_keys.pc_vault, false),
            AccountMeta::new_readonly(market_keys.coin_mint, false),
            AccountMeta::new_readonly(market_keys.coin_vault, false),
            AccountMeta::new_readonly(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: serum_crank::instruction::Initialize.data(),
    };

    sign_send_and_confirm_tx(
        &client,
        [initialize_ix].to_vec(),
        None,
        "initialize_serum_crank".to_string(),
    )?;

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
