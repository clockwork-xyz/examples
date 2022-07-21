mod utils;

use {
    anchor_lang::{
        prelude::*,
        solana_program::sysvar::{self, clock},
        system_program, InstructionData,
    },
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
    let payer = Keypair::new();
    let client = Client { client, payer };
    client.airdrop(&client.payer_pubkey(), 20 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let authority_pubkey = dca::state::Authority::pubkey(client.payer_pubkey());
    let manager_pubkey = cronos_scheduler::state::Manager::pubkey(authority_pubkey);

    // setup market
    let market_keys = setup_market(&client)?;

    // open orders account
    let mut orders = None;

    initialize(&client, authority_pubkey, manager_pubkey)?;

    delegate_funds(&client, authority_pubkey, &market_keys)?;

    init_dex_account(&client, &mut orders)?;

    init_oo_acct(&client, authority_pubkey, &market_keys, orders.unwrap())?;

    auto_swap(
        &client,
        &market_keys,
        manager_pubkey,
        authority_pubkey,
        orders.unwrap(),
    )?;

    Ok(())
}

fn setup_market(client: &Client) -> ClientResult<MarketKeys> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let dex_program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

    // generate 2 mints to list on market
    let coin_mint_pk = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    let pc_mint_pk = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    // get market listing keys
    let (listing_keys, mut ix) = gen_listing_params(
        client,
        &dex_program_id,
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
        &dex_program_id,
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

fn initialize(client: &Client, authority_pk: Pubkey, manager_pk: Pubkey) -> ClientResult<()> {
    let mut ix = Vec::new();

    // create ix for initialize ix and add to ix vec
    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new(authority_pk, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new(manager_pk, false),
        ],
        data: dca::instruction::Initialize {}.data(),
    });

    sign_send_and_confirm_tx(&client, ix, None, "initialize".to_string())?;

    Ok(())
}

fn delegate_funds(
    client: &Client,
    authority_pk: Pubkey,
    manager_pk: &MarketKeys,
) -> ClientResult<()> {
    let mut ix = Vec::new();

    // create ix for initialize ix and add to ix vec
    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pk, false),
            AccountMeta::new(manager_pk.pc_wallet_key.pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(manager_pk.pc_mint_pk, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: dca::instruction::DelegateFunds {}.data(),
    });

    sign_send_and_confirm_tx(&client, ix, None, "delegate_funds".to_string())?;

    Ok(())
}

fn init_dex_account(client: &Client, orders: &mut Option<Pubkey>) -> ClientResult<()> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let dex_program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();
    let orders_keypair;
    let mut signers = Vec::new();
    let mut ix = Vec::new();

    let orders_pk = match *orders {
        Some(pk) => pk,
        None => {
            let (orders_key, instruction) = create_dex_account(
                client,
                &dex_program_id,
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

fn init_oo_acct(
    client: &Client,
    authority_pk: Pubkey,
    market_keys: &MarketKeys,
    orders_pk: Pubkey,
) -> ClientResult<()> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let dex_program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

    let mut ix = Vec::new();

    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pk, false),
            AccountMeta::new_readonly(dex_program_id, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new_readonly(market_keys.market_pk, false),
            AccountMeta::new(orders_pk, false),
        ],
        data: dca::instruction::InitOrdersAcct {}.data(),
    });

    sign_send_and_confirm_tx(client, ix, None, "init_oo_acct".to_string())?;

    Ok(())
}

fn auto_swap(
    client: &Client,
    market_keys: &MarketKeys,
    manager_pubkey: Pubkey,
    authority_pubkey: Pubkey,
    orders_pubkey: Pubkey,
) -> ClientResult<()> {
    // temp variable local build of serum dex program id to use for bpf deployment
    let dex_program_id = Pubkey::try_from("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin").unwrap();

    let mut ix = Vec::new();

    // Derive PDAs
    let swap_queue_pk = cronos_scheduler::state::Queue::pubkey(manager_pubkey, 0);
    let swap_fee_pk = cronos_scheduler::state::Fee::pubkey(swap_queue_pk);
    let swap_task_pk = cronos_scheduler::state::Task::pubkey(swap_queue_pk, 0);

    // println!("\n");
    // println!("             authority: {}", authority_pubkey);
    // println!("                 clock: {}", clock::ID);
    // println!("                   dex: {}", dex_program_id);
    // println!("               manager: {}", manager_pubkey);
    // println!("                 payer: {}", client.payer_pubkey());
    // println!("               pc_mint: {}", market_keys.pc_mint_pk);
    // println!(
    //     "             pc_wallet: {}",
    //     market_keys.pc_wallet_key.pubkey()
    // );
    // println!("----- EXTRA ACCOUNTS -----");
    // println!("            swap_queue: {}", swap_queue_pk);
    // println!("              swap_fee: {}", swap_fee_pk);
    // println!("             swap_task: {}", swap_task_pk);
    // println!("                market: {}", market_keys.market_pk);
    // println!("            coin_vault: {}", market_keys.coin_vault_pk);
    // println!("              pc_vault: {}", market_keys.pc_vault_pk);
    // println!("                 req_q: {}", market_keys.req_q_pk);
    // println!("               event_q: {}", market_keys.event_q_pk);
    // println!("                  bids: {}", market_keys.bids_pk);
    // println!("                  asks: {}", market_keys.asks_pk);
    // println!("                orders: {}\n", orders_pubkey);
    // println!("----- DEBUG CHECK -----");
    // println!("             coin_mint: {}", market_keys.coin_mint_pk);
    // println!("            coin_vault: {}", market_keys.coin_vault_pk);
    // println!(
    //     "           coin_wallet: {}",
    //     market_keys.coin_wallet_key.pubkey()
    // );
    // println!("          vault_signer: {}", market_keys.vault_signer_pk);

    ix.push(Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority_pubkey, false),
            AccountMeta::new_readonly(clock::ID, false),
            AccountMeta::new_readonly(dex_program_id, false),
            AccountMeta::new(manager_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
            AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
            AccountMeta::new_readonly(cronos_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            // Extra Accounts
            AccountMeta::new(swap_fee_pk, false),
            AccountMeta::new(swap_queue_pk, false),
            AccountMeta::new(swap_task_pk, false),
            AccountMeta::new(market_keys.market_pk, false),
            AccountMeta::new(market_keys.coin_vault_pk, false),
            AccountMeta::new(market_keys.pc_vault_pk, false),
            AccountMeta::new(market_keys.req_q_pk, false),
            AccountMeta::new(market_keys.event_q_pk, false),
            AccountMeta::new(market_keys.bids_pk, false),
            AccountMeta::new(market_keys.asks_pk, false),
            AccountMeta::new(orders_pubkey, false),
        ],
        data: dca::instruction::AutoSwap {}.data(),
    });

    sign_send_and_confirm_tx(client, ix, None, "auto_swap".to_string())?;

    Ok(())
}
