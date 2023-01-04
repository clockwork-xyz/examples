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
    clockwork_client::{
        thread::{instruction::thread_create, state::Trigger},
        Client, ClientResult, SplToken,
    },
    clockwork_sdk::utils::PAYER_PUBKEY,
    solana_sdk::{
        instruction::Instruction,
        native_token::LAMPORTS_PER_SOL,
        signature::Keypair,
        signer::{keypair::read_keypair_file, Signer},
    },
    std::{mem::size_of, num::NonZeroU64},
    utils::*,
};

fn main() -> ClientResult<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    let thread_authority = client.payer_pubkey();

    let bob = Keypair::new();

    // airdrop a bunch bc it's expensive to setup a dex market and for all of the txs lol
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
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
    let crank_thread = clockwork_sdk::state::Thread::pubkey(thread_authority, "crank".into());

    print_explorer_link(crank, "crank".into())?;
    print_explorer_link(crank_thread, "crank_thread".into())?;

    // init serum_crank program
    initialize_serum_crank(&client, crank, crank_thread, &market_keys)?;

    // Create wallets for alice and bob
    let alice_mint_a_wallet = client.create_token_account_with_lamports(
        &client.payer_pubkey(),
        &market_keys.pc_mint,
        LAMPORTS_PER_SOL,
    )?;

    let bob_mint_b_wallet = client.create_token_account_with_lamports(
        &bob.pubkey(),
        &market_keys.coin_mint,
        LAMPORTS_PER_SOL,
    )?;

    // mint to alice and bob's wallets
    client.mint_to(
        client.payer(),
        &market_keys.pc_mint,
        &alice_mint_a_wallet.pubkey(),
        1_000_000_000_000_000,
        9,
    )?;

    client.mint_to(
        client.payer(),
        &market_keys.coin_mint,
        &bob_mint_b_wallet.pubkey(),
        1_000_000_000_000_000,
        9,
    )?;

    // initialize open order accounts for alice and bob
    let mut oo_account_alice = None;

    init_open_orders_account(
        &client,
        &openbook_dex_pk(),
        client.payer(),
        &market_keys,
        &mut oo_account_alice,
    )?;

    let mut oo_account_bob = None;

    init_open_orders_account(
        &client,
        &openbook_dex_pk(),
        &bob,
        &market_keys,
        &mut oo_account_bob,
    )?;

    // place orders
    for _ in 0..5 {
        place_order(
            &client,
            &openbook_dex_pk(),
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
            &openbook_dex_pk(),
            client.payer(),
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
    crank_thread: Pubkey,
    market_keys: &MarketKeys,
) -> ClientResult<()> {
    client.airdrop(&crank_thread, LAMPORTS_PER_SOL)?;

    // destructor struct for convenience
    let MarketKeys {
        event_q,
        market,
        pc_mint,
        pc_vault,
        coin_mint,
        coin_vault,
        ..
    } = *market_keys;

    // define initialize ix
    let initialize_ix = Instruction {
        program_id: serum_crank::ID,
        accounts: vec![
            AccountMeta::new(crank, false),
            AccountMeta::new_readonly(openbook_dex_pk(), false),
            AccountMeta::new_readonly(event_q, false),
            AccountMeta::new_readonly(market, false),
            AccountMeta::new_readonly(pc_mint, false),
            AccountMeta::new_readonly(pc_vault, false),
            AccountMeta::new_readonly(coin_mint, false),
            AccountMeta::new_readonly(coin_vault, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: serum_crank::instruction::Initialize.data(),
    };

    // create thread with read events ix
    let thread_create = thread_create(
        client.payer_pubkey(),
        "crank".into(),
        Instruction {
            program_id: serum_crank::ID,
            accounts: vec![
                AccountMeta::new(crank.key(), false),
                AccountMeta::new(crank_thread.key(), true),
                AccountMeta::new_readonly(openbook_dex_pk(), false),
                AccountMeta::new_readonly(event_q, false),
                AccountMeta::new_readonly(market, false),
                AccountMeta::new_readonly(pc_vault, false),
                AccountMeta::new_readonly(coin_vault, false),
                AccountMeta::new(PAYER_PUBKEY, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: serum_crank::instruction::ReadEvents.data(),
        }
        .into(),
        client.payer_pubkey(),
        crank_thread,
        Trigger::Account {
            address: event_q,
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

fn default_client() -> Client {
    #[cfg(not(feature = "localnet"))]
    let host = "https://api.devnet.solana.com";
    #[cfg(feature = "localnet")]
    let host = "http://localhost:8899";

    let config_file = solana_cli_config::CONFIG_FILE.as_ref().unwrap().as_str();
    let config = solana_cli_config::Config::load(config_file).unwrap();
    let payer = read_keypair_file(&config.keypair_path).unwrap();
    Client::new(payer, host.into())
}
