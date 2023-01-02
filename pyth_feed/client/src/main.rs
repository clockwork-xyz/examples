use {
    anchor_lang::{prelude::*, solana_program::system_program, InstructionData},
    clockwork_client::{
        thread::{
            ID as thread_program_ID,
            state::{Thread},
        },
        Client, ClientResult,
    },
    clockwork_utils::{explorer::Explorer},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL,
        signature::Keypair, signature::read_keypair_file,
        transaction::Transaction,
    },
    std::str::FromStr,
};


fn main() -> ClientResult<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    client.airdrop(&client.payer_pubkey(), 1 * LAMPORTS_PER_SOL)?;

    // create thread that listens for account changes for a pyth pricing feed
    create_feed(&client)?;

    Ok(())
}

fn create_feed(client: &Client) -> ClientResult<()> {
    let feed_pubkey = pyth_feed::state::Feed::pubkey(client.payer_pubkey());
    let feed_thread_pubkey = Thread::pubkey(feed_pubkey, "feed".into());

    // SOL/USD price feed: https://pyth.network/price-feeds/crypto-sol-usd?cluster=mainnet-beta
    // copied account to test validator using https://book.anchor-lang.com/anchor_references/anchor-toml_reference.html#testvalidatorclone
    #[cfg(feature = "localnet")]
        let sol_usd_pubkey = Pubkey::from_str("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap();
    #[cfg(not(feature = "localnet"))] // for devnet
        let sol_usd_pubkey = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();

    println!(
        "thread: 🔗 {}",
        explorer().thread_url(feed_thread_pubkey, thread_program_ID)
    );

    // airdrop thread
    std::thread::sleep(std::time::Duration::from_secs(1));
    client.airdrop(&feed_thread_pubkey, 1 * LAMPORTS_PER_SOL)?;

    let create_feed_ix = Instruction {
        program_id: pyth_feed::ID,
        accounts: vec![
            AccountMeta::new_readonly(thread_program_ID, false),
            AccountMeta::new(feed_pubkey, false),
            AccountMeta::new(feed_thread_pubkey, false),
            AccountMeta::new_readonly(sol_usd_pubkey, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: pyth_feed::instruction::CreateFeed {}.data(),
    };
    sign_send_and_confirm_tx(
        &client,
        [create_feed_ix].to_vec(),
        None,
        "create_feed".into(),
    )?;
    Ok(())
}

pub fn sign_send_and_confirm_tx(
    client: &Client,
    ix: Vec<Instruction>,
    signers: Option<Vec<&Keypair>>,
    label: String,
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
            // Eventually also use EXPLORER.clockwork instead of EXPLORER.solana, so ppl don't have to use two explorers
            "{} tx: ✅ {}",
            label,
            explorer().tx_url(sig)
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
}

fn explorer() -> Explorer {
    #[cfg(feature = "localnet")]
    return Explorer::custom("http://localhost:8899".to_string());
    #[cfg(not(feature = "localnet"))]
    Explorer::devnet()
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
