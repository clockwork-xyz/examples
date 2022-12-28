use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_client::{
        thread::{instruction::thread_create, state::Trigger},
        Client, ClientResult,
    },
    solana_sdk::{instruction::Instruction, signature::Keypair, transaction::Transaction},
    std::str::FromStr,
};

#[cfg(not(feature = "mainnet"))]
fn main() -> ClientResult<()> {
    // Create Client
    let payer = Keypair::new();
    let client = Client::new(payer, "https://api.devnet.solana.com".into());
    client.airdrop(
        &client.payer_pubkey(),
        2 * solana_sdk::native_token::LAMPORTS_PER_SOL,
    )?;

    // SOL/USD price feed
    let sol_usd_pubkey = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();

    create_feed(
        &client,
        sol_usd_pubkey,
        Cluster::Devnet,
        "sol_usd_stat".into(),
    )?;

    Ok(())
}

#[cfg(feature = "mainnet")]
fn main() -> ClientResult<()> {
    use {solana_sdk::signature::read_keypair_file, std::env};

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Keypair path not provided");
    }

    let payer = read_keypair_file(&args[1]).expect("invalid keypair path");
    let client = Client::new(payer, "https://api.mainnet-beta.solana.com".into());

    // SOL/USD price feed
    let sol_usd_pubkey = Pubkey::from_str("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap();

    create_feed(
        &client,
        sol_usd_pubkey,
        Cluster::Mainnet,
        "sol_usd_stat".into(),
    )?;

    Ok(())
}

fn create_feed(
    client: &Client,
    price_feed_pubkey: Pubkey,
    cluster: Cluster,
    thread_id: &str,
) -> ClientResult<()> {
    let lookback_window: i64 = 7203; // seconds
    let stat_pubkey =
        pyth_stats::state::Stat::pubkey(price_feed_pubkey, client.payer_pubkey(), lookback_window);
    let stat_thread_pubkey =
        clockwork_client::thread::state::Thread::pubkey(client.payer_pubkey(), thread_id.into());
    let avg_buffer_pubkey = pyth_stats::state::AvgBuffer::pubkey(stat_pubkey);
    let price_buffer_pubkey = pyth_stats::state::PriceBuffer::pubkey(stat_pubkey);
    let time_series_pubkey = pyth_stats::state::TimeSeries::pubkey(stat_pubkey);

    print_explorer_link(stat_pubkey, "stat account".into(), cluster)?;
    print_explorer_link(stat_thread_pubkey, "stat_thread".into(), cluster)?;

    // airdrop thread
    #[cfg(not(feature = "mainnet"))]
    client.airdrop(
        &stat_thread_pubkey,
        2 * solana_sdk::native_token::LAMPORTS_PER_SOL,
    )?;

    let initialize_ix = Instruction {
        program_id: pyth_stats::ID,
        accounts: vec![
            AccountMeta::new(avg_buffer_pubkey, false),
            AccountMeta::new(price_buffer_pubkey, false),
            AccountMeta::new_readonly(price_feed_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new(stat_pubkey, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(time_series_pubkey, false),
        ],
        data: pyth_stats::instruction::Initialize { lookback_window }.data(),
    };

    let create_thread_ix = thread_create(
        client.payer_pubkey(),
        thread_id.into(),
        Instruction {
            program_id: pyth_stats::ID,
            accounts: vec![
                AccountMeta::new(avg_buffer_pubkey, false),
                AccountMeta::new(price_buffer_pubkey, false),
                AccountMeta::new_readonly(price_feed_pubkey, false),
                AccountMeta::new(stat_pubkey, false),
                AccountMeta::new(stat_thread_pubkey, true),
                AccountMeta::new(time_series_pubkey, false),
            ],
            data: pyth_stats::instruction::Calc {}.data(),
        }
        .into(),
        client.payer_pubkey(),
        stat_thread_pubkey,
        Trigger::Cron {
            schedule: "*/10 * * * * *".into(),
            skippable: true,
        },
    );

    sign_send_and_confirm_tx(
        &client,
        [initialize_ix, create_thread_ix].to_vec(),
        None,
        "initialize and create thread".into(),
        cluster,
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

pub fn sign_send_and_confirm_tx(
    client: &Client,
    ix: Vec<Instruction>,
    signers: Option<Vec<&Keypair>>,
    label: String,
    cluster: Cluster,
) -> ClientResult<()> {
    let mut tx;

    match signers {
        Some(signer_keypairs) => {
            tx = Transaction::new_signed_with_payer(
                &ix,
                Some(&client.payer_pubkey()),
                &signer_keypairs,
                client.get_latest_blockhash().unwrap(),
            );
        }
        None => {
            tx = Transaction::new_with_payer(&ix, Some(&client.payer_pubkey()));
        }
    }

    tx.sign(&[client.payer()], client.latest_blockhash().unwrap());

    // Send and confirm tx
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster={}",
            label,
            sig,
            cluster.value()
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
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
