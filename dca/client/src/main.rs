use std::mem::size_of;

use anchor_spl::dex::serum_dex::{self, state::OpenOrders};
use solana_sdk::{sysvar::clock, transaction::Transaction};

mod utils;

use {
    anchor_lang::{prelude::*, solana_program::sysvar, system_program, InstructionData},
    anchor_spl::{associated_token, dex::serum_dex::instruction::initialize_market, token},
    debug_print::debug_println,
    serum_common::client::rpc::mint_to_new_account,
    solana_client_helpers::{Client, ClientResult, RpcClient, SplToken},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer,
    },
    utils::*,
};

fn main() -> ClientResult<()> {
    // Create Client
    let client = RpcClient::new("http://localhost:8899");
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), 20 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let authority_pubkey = dca::state::Authority::pda().0;
    let manager_pubkey = cronos_scheduler::state::Manager::pda(authority_pubkey).0;

    // setup market
    let market_keys = setup_market(&client)?;

    initialize(&client, authority_pubkey, manager_pubkey)?;

    delegate_funds(&client, authority_pubkey, manager_pubkey, &market_keys)?;

    let mut orders = None;

    init_open_order_account(
        &client,
        authority_pubkey,
        manager_pubkey,
        &market_keys,
        &mut orders,
    )?;

    debug_println!("manager: {}", manager_pubkey);
    debug_println!("oo: {}", orders.unwrap());

    // auto_swap(&client, &market_keys, &orders)?;

    // temp for testing
    // swap(&client, &market_keys, &orders)?;

    Ok(())
}

fn setup_market(client: &Client) -> ClientResult<MarketKeys> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

    // generate 2 mints to list on market
    let coin_mint_pk = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    debug_println!("coin mint: {}", coin_mint_pk);

    let pc_mint_pk = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    debug_println!("pc mint: {}", pc_mint_pk);

    // get market listing keys
    let (listing_keys, mut ix) = gen_listing_params(
        client,
        &program_id,
        &client.payer_pubkey(),
        &coin_mint_pk,
        &pc_mint_pk,
    )?;

    // destructuring market listing keys
    let ListingKeys {
        market_key,
        req_q_key,
        event_q_key,
        bids_key,
        asks_key,
        vault_signer_pk,
        vault_signer_nonce,
    } = listing_keys;

    // create ata vaults for the respective mints
    let coin_vault_pk =
        client.create_associated_token_account(&client.payer(), &vault_signer_pk, &coin_mint_pk)?;

    let pc_vault_pk =
        client.create_associated_token_account(&client.payer(), &vault_signer_pk, &pc_mint_pk)?;

    // get the init market ix
    let init_market_ix = initialize_market(
        &market_key.pubkey(),
        &program_id,
        &coin_mint_pk,
        &pc_mint_pk,
        &coin_vault_pk,
        &pc_vault_pk,
        None,
        None,
        &bids_key.pubkey(),
        &asks_key.pubkey(),
        &req_q_key.pubkey(),
        &event_q_key.pubkey(),
        1_000_000,
        10_000,
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
    let coin_wallet = mint_to_new_account(
        &client,
        &client.payer(),
        &client.payer(),
        &coin_mint_pk,
        1_000_000_000_000_000,
    )
    .unwrap();

    let pc_wallet = mint_to_new_account(
        &client,
        &client.payer(),
        &client.payer(),
        &pc_mint_pk,
        1_000_000_000_000_000,
    )
    .unwrap();

    debug_println!("Listing market: {} ...", market_key.pubkey());

    Ok(MarketKeys {
        market_pk: market_key.pubkey(),
        req_q_pk: req_q_key.pubkey(),
        event_q_pk: event_q_key.pubkey(),
        bids_pk: bids_key.pubkey(),
        asks_pk: asks_key.pubkey(),
        coin_mint_pk,
        coin_vault_pk,
        pc_mint_pk,
        pc_vault_pk,
        vault_signer_pk,
        pc_wallet_key: pc_wallet,
        coin_wallet_key: coin_wallet,
    })
}

fn initialize(
    client: &Client,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
) -> ClientResult<()> {
    let mut ix = Vec::new();

    // create ix for initialize ix and add to ix vec
    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new(authority_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new(manager_pubkey, false),
        ],
        data: dca::instruction::Initialize {}.data(),
    });

    sign_send_and_confirm_tx(&client, ix, None, "initialize".to_string())?;

    Ok(())
}

fn delegate_funds(
    client: &Client,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
    market_keys: &MarketKeys,
) -> ClientResult<()> {
    let mut ix = Vec::new();

    // debug_println!("{}", authority_pubkey);
    // debug_println!("{}", manager_pubkey);
    // debug_println!("{}", market_keys.pc_wallet_key.pubkey());
    // debug_println!("{}", market_keys.pc_mint_pk);

    // create ix for initialize ix and add to ix vec
    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new(manager_pubkey, false),
            AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: dca::instruction::DelegateFunds {}.data(),
    });

    sign_send_and_confirm_tx(&client, ix, None, "delegate_funds".to_string())?;

    Ok(())
}

fn init_open_order_account(
    client: &Client,
    authority_pubkey: Pubkey,
    manager_pubkey: Pubkey,
    market_keys: &MarketKeys,
    orders: &mut Option<Pubkey>,
) -> ClientResult<()> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();
    let orders_keypair;
    let mut signers = Vec::new();
    let mut ix = Vec::new();

    let orders_pubkey = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) = create_dex_account(
                client,
                &program_id,
                &client.payer_pubkey(),
                size_of::<OpenOrders>(),
            )?;
            orders_keypair = orders_key;
            signers.push(&orders_keypair);
            ix.push(instruction);
            orders_keypair.pubkey()
        }
    };

    *orders = Some(orders_pubkey);

    signers.push(client.payer());

    sign_send_and_confirm_tx(client, ix, Some(signers), "create_dex_acct".to_string())?;

    let mut ix2 = Vec::new();

    debug_println!("{}", authority_pubkey);
    debug_println!("{}", program_id);
    debug_println!("{}", manager_pubkey);
    debug_println!("{}", orders_pubkey);
    debug_println!("{}", client.payer_pubkey());
    debug_println!("{}", market_keys.market_pk);

    ix2.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new_readonly(manager_pubkey, false),
            AccountMeta::new(orders_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new_readonly(market_keys.market_pk, false),
        ],
        data: dca::instruction::InitOpenOrderAccount {}.data(),
    });

    sign_send_and_confirm_tx(client, ix2, None, "init_open_order_account".to_string())?;

    Ok(())
}

fn _auto_swap(
    client: &Client,
    market_keys: &MarketKeys,
    orders: &Option<Pubkey>,
    manager_pubkey: Pubkey,
    authority_pubkey: Pubkey,
) -> ClientResult<()> {
    let mut ix = Vec::new();
    let program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

    // Derive PDAs
    let swap_queue_pubkey = cronos_scheduler::state::Queue::pda(manager_pubkey, 0).0;
    let swap_fee_pubkey = cronos_scheduler::state::Fee::pda(swap_queue_pubkey).0;
    let swap_task_pubkey = cronos_scheduler::state::Task::pda(swap_queue_pubkey, 0).0;

    debug_println!("\n");
    debug_println!("              authority: {}", authority_pubkey);
    debug_println!("                  clock: {}", clock::ID);
    debug_println!("                   dex: {}", program_id);
    debug_println!("                 payer: {}", client.payer_pubkey());
    debug_println!("   market_keys.pc_mint: {}", market_keys.pc_mint_pk);
    debug_println!(
        "             pc_wallet: {}",
        market_keys.pc_wallet_key.pubkey()
    );
    debug_println!("----- EXTRA ACCOUNTS -----");
    debug_println!("                market: {}", market_keys.market_pk);
    debug_println!("            coin_vault: {}", market_keys.coin_vault_pk);
    debug_println!("              pc_vault: {}", market_keys.pc_vault_pk);
    debug_println!("                 req_q: {}", market_keys.req_q_pk);
    debug_println!("               event_q: {}", market_keys.event_q_pk);
    debug_println!("                  bids: {}", market_keys.bids_pk);
    debug_println!("                  asks: {}", market_keys.asks_pk);
    debug_println!("                orders: {}\n", orders.unwrap());

    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new_readonly(clock::ID, false),
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new_readonly(manager_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
            AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            // Extra Accounts
            AccountMeta::new(swap_fee_pubkey, false),
            AccountMeta::new(swap_queue_pubkey, false),
            AccountMeta::new(swap_task_pubkey, false),
            AccountMeta::new(market_keys.market_pk, false),
            AccountMeta::new(market_keys.coin_vault_pk, false),
            AccountMeta::new(market_keys.pc_vault_pk, false),
            AccountMeta::new(market_keys.req_q_pk, false),
            AccountMeta::new(market_keys.event_q_pk, false),
            AccountMeta::new(market_keys.bids_pk, false),
            AccountMeta::new(market_keys.asks_pk, false),
            AccountMeta::new(orders.unwrap(), false),
        ],
        data: dca::instruction::AutoSwap {}.data(),
    });

    sign_send_and_confirm_tx(client, ix, None, "swap".to_string())?;

    Ok(())
}

// fn _swap(client: &Client, market_keys: &MarketKeys, orders: &Option<Pubkey>) -> ClientResult<()> {
//     let mut ix = Vec::new();
//     let program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

//     debug_println!("\n");
//     debug_println!("      associated_token: {}", associated_token::ID);
//     debug_println!("            program_id: {}", program_id);
//     debug_println!("                 payer: {}", client.payer_pubkey());
//     debug_println!(
//         "             pc_wallet: {}",
//         market_keys.pc_wallet_key.pubkey()
//     );
//     debug_println!("   market_keys.pc_mint: {}", market_keys.pc_mint_pk);
//     debug_println!("         token program: {}", token::ID);
//     debug_println!("                  rent: {}", sysvar::rent::ID);
//     debug_println!("                  rent: {}", system_program::ID);
//     debug_println!("----- EXTRA ACCOUNTS -----");
//     debug_println!("                market: {}", market_keys.market_pk);
//     debug_println!("            coin_vault: {}", market_keys.coin_vault_pk);
//     debug_println!("              pc_vault: {}", market_keys.pc_vault_pk);
//     debug_println!("                 req_q: {}", market_keys.req_q_pk);
//     debug_println!("               event_q: {}", market_keys.event_q_pk);
//     debug_println!("                  bids: {}", market_keys.bids_pk);
//     debug_println!("                  asks: {}", market_keys.asks_pk);
//     debug_println!("                orders: {}\n", orders.unwrap());

//     ix.push(Instruction {
//         program_id: dca::ID,
//         accounts: vec![
//             AccountMeta::new_readonly(associated_token::ID, false),
//             AccountMeta::new_readonly(program_id, false),
//             AccountMeta::new(client.payer_pubkey(), true),
//             AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
//             AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
//             AccountMeta::new_readonly(token::ID, false),
//             AccountMeta::new_readonly(sysvar::rent::ID, false),
//             AccountMeta::new_readonly(system_program::ID, false),
//             // Extra Accounts
//             AccountMeta::new(market_keys.market_pk, false),
//             AccountMeta::new(market_keys.coin_vault_pk, false),
//             AccountMeta::new(market_keys.pc_vault_pk, false),
//             AccountMeta::new(market_keys.req_q_pk, false),
//             AccountMeta::new(market_keys.event_q_pk, false),
//             AccountMeta::new(market_keys.bids_pk, false),
//             AccountMeta::new(market_keys.asks_pk, false),
//             AccountMeta::new(orders.unwrap(), false),
//         ],
//         data: dca::instruction::Swap {}.data(),
//     });

//     sign_send_and_confirm_tx(client, ix, None, "swap".to_string())?;

//     Ok(())
// }

// *orders = Some(open_order_pubkey);
// let open_order_pubkey = Pubkey::find_program_address(
//     &[b"open_order", client.payer_pubkey().as_ref()],
//     &program_id,
// )
// .0;
// debug_println!("{}", authority_pubkey);
// debug_println!("{}", program_id);
// debug_println!("{}", manager_pubkey);
// debug_println!("{}", open_order_pubkey);
// debug_println!("{}", client.payer_pubkey());
// debug_println!("{}", market_keys.market_pk);

// ix.push(Instruction {
//     program_id: dca::ID,
//     accounts: vec![
//         AccountMeta::new_readonly(authority_pubkey, false),
//         AccountMeta::new_readonly(program_id, false),
//         AccountMeta::new_readonly(manager_pubkey, false),
//         AccountMeta::new(open_order_pubkey, false),
//         AccountMeta::new(client.payer_pubkey(), true),
//         AccountMeta::new_readonly(sysvar::rent::ID, false),
//         AccountMeta::new_readonly(system_program::ID, false),
//         // Extra accounts
//         AccountMeta::new_readonly(market_keys.market_pk, false),
//     ],
//     data: dca::instruction::InitOpenOrderAccount {}.data(),
// });
// sign_send_and_confirm_tx(client, ix, None, "init_open_order_account".to_string())?;

// ix.push(
//     anchor_spl::dex::serum_dex::instruction::init_open_orders(
//         &program_id,
//         &orders_pubkey,
//         &client.payer_pubkey(),
//         &market_keys.market_pk,
//         None,
//     )
//     .unwrap(),
// );

// signers.push(client.payer());

// sign_send_and_confirm_tx(client, ix, Some(signers), "init_open_orders".to_string())?;
// sign_send_and_confirm_tx(client, ix, Some(signers), "init_open_orders".to_string())?;

// debug_println!("{}", authority_pubkey);
// debug_println!("{}", program_id);
// debug_println!("{}", manager_pubkey);
// debug_println!("{}", orders_pubkey);
// debug_println!("{}", client.payer_pubkey());
// debug_println!("{}", market_keys.market_pk);

// ix.push(Instruction {
//     program_id: dca::ID,
//     accounts: vec![
//         AccountMeta::new_readonly(authority_pubkey, false),
//         AccountMeta::new_readonly(program_id, false),
//         AccountMeta::new_readonly(manager_pubkey, false),
//         AccountMeta::new(orders_pubkey, false),
//         AccountMeta::new(client.payer_pubkey(), true),
//         AccountMeta::new_readonly(sysvar::rent::ID, false),
//         AccountMeta::new_readonly(system_program::ID, false),
//         // Extra accounts
//         AccountMeta::new_readonly(market_keys.market_pk, false),
//     ],
//     data: dca::instruction::InitOpenOrderAccount {}.data(),
// });

// signers.push(client.payer());

// sign_send_and_confirm_tx(client, ix, None, "init_open_order_account".to_string())?;

// ix.push(
//     anchor_spl::dex::serum_dex::instruction::init_open_orders(
//         &program_id,
//         &orders_pubkey,
//         &client.payer_pubkey(),
//         &market_keys.market_pk,
//         Some(&manager_pubkey),
//     )
//     .unwrap(),
// );

// let mut setup_transaction = Transaction::new_with_payer(&ix, Some(&client.payer_pubkey()));

// signers.push(client.payer());

// setup_transaction.partial_sign(&signers, client.get_latest_blockhash().unwrap());

// client.process_transaction(&setup_transaction)?;
