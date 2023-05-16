#![allow(unused)]

mod client;
use client::*;

use {
    anchor_lang::{
        prelude::*,
        solana_program::{
            instruction::Instruction,
            system_program,
        },
        InstructionData,
        ToAccountMetas,
    },
    anyhow::Result,
    bincode::serialize,
    clockwork_thread_program::state::{
        LookupTables,
        Thread,
        Trigger,
        PAYER_PUBKEY,
    },
    serde_json::json,
    solana_address_lookup_table_program::{
        instruction::{
            create_lookup_table,
            extend_lookup_table,
        },
        state::AddressLookupTable,
    },
    solana_client::{
        rpc_config::RpcSendTransactionConfig,
        rpc_request::RpcRequest,
    },
    solana_sdk::{
        address_lookup_table_account::AddressLookupTableAccount,
        commitment_config::{
            CommitmentConfig,
            CommitmentLevel,
        },
        message::{
            v0,
            VersionedMessage,
        },
        native_token::LAMPORTS_PER_SOL,
        signature::{
            read_keypair_file,
            Keypair,
            Signature,
            Signer,
        },
        signers::Signers,
        slot_history::Slot,
        system_instruction,
        transaction::{
            Transaction,
            VersionedTransaction,
        },
    },
    solana_transaction_status::UiTransactionEncoding,
    std::{
        str::FromStr,
        thread,
        time,
    },
};

fn main() -> Result<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    let app_localnet_simul_pk =
        Pubkey::from_str("GuJVu6wky7zeVaPkGaasC5vx1eVoiySbEv7UFKZAu837").unwrap();
    client.airdrop(&app_localnet_simul_pk, LAMPORTS_PER_SOL)?;

    println!("Create the address lookup table");
    let recent_slot = client
        .get_slot_with_commitment(CommitmentConfig::finalized())
        .unwrap();
    let lut_auth = client.payer_pubkey();
    let (create_ix, lut) = solana_address_lookup_table_program::instruction::create_lookup_table(
        lut_auth,
        client.payer_pubkey(),
        recent_slot,
    );
    let latest_blockhash = client.get_latest_blockhash().unwrap();
    client
        .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&client.payer_pubkey()),
            &[client.payer()],
            latest_blockhash,
        ))
        .unwrap();

    // Our Target ixs
    let mut ixs = Vec::new();
    let mut keys: Vec<Pubkey> = Vec::new();
    for i in 0..50 {
        let kp = Keypair::new();
        let target_ix = system_instruction::transfer(&PAYER_PUBKEY, &kp.pubkey(), LAMPORTS_PER_SOL);
        keys.push(kp.pubkey());
        ixs.push(target_ix);
    }

    println!("Loop to extend the address lookup table");
    let mut signature = Signature::default();
    let latest_blockhash = client.get_latest_blockhash().unwrap();
    for keys in keys.chunks(20) {
        let extend_ix = solana_address_lookup_table_program::instruction::extend_lookup_table(
            lut,
            lut_auth,
            Some(client.payer_pubkey()),
            keys.into(),
        );

        signature = client
            .send_and_confirm_transaction(&Transaction::new_signed_with_payer(
                &[extend_ix],
                Some(&client.payer_pubkey()),
                &[&client.payer],
                latest_blockhash,
            ))
            .unwrap();
    }
    client
        .confirm_transaction_with_spinner(
            &signature,
            &latest_blockhash,
            CommitmentConfig::finalized(),
        )
        .unwrap();

    // Out Target ix
    //let target_ix = system_instruction::transfer(
    //    &client.payer_pubkey(),
    //    &kp.pubkey(),
    //    LAMPORTS_PER_SOL,
    //);

    // Thread stuff
    let ts = chrono::Local::now();
    let thread_id = format!("{}_{}", "lutrs", ts.format("%d_%H:%M:%S"));
    let thread_auth = client.payer_pubkey();
    let thread = Thread::pubkey(thread_auth, thread_id.clone().into());

    let thread_create_ix = Instruction {
        program_id: clockwork_thread_program::ID,
        accounts: clockwork_thread_program::accounts::ThreadCreate {
            authority: client.payer_pubkey(),
            payer: client.payer_pubkey(),
            system_program: system_program::ID,
            thread,
        }
        .to_account_metas(Some(false)),
        data: clockwork_thread_program::instruction::ThreadCreate {
            amount: LAMPORTS_PER_SOL,
            id: thread_id.into(),
            instructions: ixs.iter().map(|e| e.clone().into()).collect(),
            trigger: Trigger::Cron {
                schedule: "*/10 * * * * * *".into(),
                skippable: true,
            },
        }
        .data(),
    };
    println!("thread {:#?}", thread);

    // Add LookupTables to Thread
    let thread_lut = LookupTables::pubkey(thread_auth, thread);
    let create_thread_lut_ix = Instruction {
        program_id: clockwork_thread_program::ID,
        accounts: clockwork_thread_program::accounts::LookupTablesCreate {
            authority: client.payer_pubkey(),
            payer: client.payer_pubkey(),
            system_program: system_program::ID,
            thread,
            lookup_tables: thread_lut,
        }
        .to_account_metas(Some(false)),
        data: clockwork_thread_program::instruction::ThreadLookupTablesCreate {
            address_lookup_tables: vec![lut],
        }
        .data(),
    };

    let ixs = [thread_create_ix, create_thread_lut_ix];
    //let sig = client.send_and_confirm(&ixs, &[&client.payer])?;
    //println!("✍️s: https://explorer.solana.com/tx/{}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899", sig);

    println!("Create signed legacy tx");
    let mut signers = vec![&client.payer];
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&client.payer_pubkey()),
        &signers,
        latest_blockhash,
    );
    let serialized_tx = serialize(&tx).unwrap();
    println!("This legacy serialized tx is {} bytes", serialized_tx.len());

    println!("Wait some arbitrary amount of time to please the address lookup table");
    thread::sleep(time::Duration::from_secs(3));

    println!("Create versioned tx");
    let versioned_tx = create_tx_with_address_table_lookup(&client, &ixs, lut, &signers).unwrap();
    let serialized_versioned_tx = serialize(&versioned_tx).unwrap();
    println!(
        "The serialized versioned tx is {} bytes",
        serialized_versioned_tx.len()
    );
    let serialized_encoded = base64::encode(serialized_versioned_tx);
    let config = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(CommitmentLevel::Processed),
        encoding: Some(UiTransactionEncoding::Base64),
        ..RpcSendTransactionConfig::default()
    };

    let signature = client
        .send::<String>(
            RpcRequest::SendTransaction,
            json!([serialized_encoded, config]),
        )
        .unwrap();
    println!("✍️s: https://explorer.solana.com/tx/{}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899", signature);
    client
        .confirm_transaction_with_commitment(
            &Signature::from_str(&signature).unwrap(),
            CommitmentConfig::finalized(),
        )
        .unwrap();

    thread::sleep(time::Duration::from_secs(2)); // Not sure why this is required while commitments are compatible

    // We craft our own getTransaction as RpcClient doesn't support v0
    let rqclient = reqwest::blocking::Client::new();
    let res = rqclient
        .post("http://localhost:8899/")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                {"encoding": "json", "commitment": "confirmed", "maxSupportedTransactionVersion": 0}
            ]
        }))
        .send()
        .unwrap();
    println!("{:?}", res.text().unwrap());

    let raw_account = client.get_account(&lut)?;
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data)?;
    let addresses = address_lookup_table.addresses;
    println!("keys: {:#?}", keys);
    println!("lut: {}", lut);
    println!("{:#?}", addresses);
    let latest_blockhash = client.get_latest_blockhash().unwrap();
    for k in addresses.as_ref().iter() {
        let bal = client.get_balance(k)?;
        println!("{}: {}", k, bal);
    }

    Ok(())
}

fn create_tx_with_address_table_lookup<T: Signers>(
    client: &Client,
    instructions: &[Instruction],
    address_lookup_table_key: Pubkey,
    signers: &T,
) -> Result<VersionedTransaction> {
    let raw_account = client.get_account(&address_lookup_table_key)?;
    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data)?;
    let address_lookup_table_account = AddressLookupTableAccount {
        key: address_lookup_table_key,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let blockhash = client.get_latest_blockhash()?;
    let tx = VersionedTransaction::try_new(
        VersionedMessage::V0(v0::Message::try_compile(
            &client.payer_pubkey(),
            instructions,
            &[address_lookup_table_account],
            blockhash,
        )?),
        signers,
    )?;

    assert!(!tx.message.address_table_lookups().unwrap().is_empty());
    Ok(tx)
}

fn default_client() -> Client {
    let config_file = solana_cli_config::CONFIG_FILE.as_ref().unwrap().as_str();
    let config = solana_cli_config::Config::load(config_file).unwrap();
    let keypair = read_keypair_file(&config.keypair_path).unwrap();
    Client::new(keypair, config.json_rpc_url)
}
