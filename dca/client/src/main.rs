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
    let client = RpcClient::new("https://api.devnet.solana.com");
    let payer = Keypair::new();
    let client = Client { client, payer };

    // airdrop a bunch bc it's expensive to setup a dex market and for all of the txs lol
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let authority = dca::state::Authority::pubkey(client.payer_pubkey());
    let queue = clockwork_scheduler::state::Queue::pubkey(authority, "dca_queue".to_string());
    let task = clockwork_scheduler::state::Task::pubkey(queue, 0);

    // setup market
    let market_keys = setup_market(&client)?;

    // open orders account
    let mut orders = None;

    create_queue(&client, authority, queue)?;

    delegate_funds(&client, authority, &market_keys)?;

    init_dex_account(&client, &mut orders)?;

    create_orders(&client, authority, &market_keys, orders.unwrap())?;

    create_task(
        &client,
        &market_keys,
        queue,
        task,
        authority,
        orders.unwrap(),
    )?;

    Ok(())
}

fn setup_market(client: &Client) -> ClientResult<MarketKeys> {
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
        &anchor_spl::dex::ID,
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
        &anchor_spl::dex::ID,
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

fn create_queue(client: &Client, authority: Pubkey, queue: Pubkey) -> ClientResult<()> {
    // create ix
    let ix = Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: dca::instruction::CreateQueue {}.data(),
    };

    sign_send_and_confirm_tx(&client, [ix].to_vec(), None, "create_queue".to_string())?;

    Ok(())
}

fn delegate_funds(
    client: &Client,
    authority: Pubkey,
    market_keys: &MarketKeys,
) -> ClientResult<()> {
    // create ix
    let ix = Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: dca::instruction::DelegateFunds {}.data(),
    };

    sign_send_and_confirm_tx(&client, [ix].to_vec(), None, "delegate_funds".to_string())?;

    Ok(())
}

fn init_dex_account(client: &Client, orders: &mut Option<Pubkey>) -> ClientResult<()> {
    let orders_keypair;
    let mut signers = Vec::new();
    let mut ix = Vec::new();

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

fn create_orders(
    client: &Client,
    authority: Pubkey,
    market_keys: &MarketKeys,
    orders: Pubkey,
) -> ClientResult<()> {
    let ix = Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new_readonly(anchor_spl::dex::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            // Extra accounts
            AccountMeta::new_readonly(market_keys.market_pk, false),
            AccountMeta::new(orders, false),
        ],
        data: dca::instruction::CreateOrders {}.data(),
    };

    sign_send_and_confirm_tx(client, [ix].to_vec(), None, "create_orders".to_string())?;

    Ok(())
}

fn create_task(
    client: &Client,
    market_keys: &MarketKeys,
    queue: Pubkey,
    task: Pubkey,
    authority: Pubkey,
    orders: Pubkey,
) -> ClientResult<()> {
    let ix = Instruction {
        program_id: dca::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new_readonly(anchor_spl::dex::ID, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(market_keys.pc_mint_pk, false),
            AccountMeta::new(market_keys.pc_wallet_key.pubkey(), false),
            AccountMeta::new(queue, false),
            AccountMeta::new_readonly(clockwork_scheduler::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(task, false),
            AccountMeta::new_readonly(token::ID, false),
            // Extra Accounts
            AccountMeta::new(market_keys.market_pk, false),
            AccountMeta::new(market_keys.coin_vault_pk, false),
            AccountMeta::new(market_keys.pc_vault_pk, false),
            AccountMeta::new(market_keys.req_q_pk, false),
            AccountMeta::new(market_keys.event_q_pk, false),
            AccountMeta::new(market_keys.bids_pk, false),
            AccountMeta::new(market_keys.asks_pk, false),
            AccountMeta::new(orders, false),
        ],
        data: dca::instruction::CreateTask {}.data(),
    };

    sign_send_and_confirm_tx(client, [ix].to_vec(), None, "create_task".to_string())?;

    Ok(())
}
