use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            native_token::LAMPORTS_PER_SOL,
            system_program,
        },
        InstructionData,
    },
    clockwork_client::{thread::state::Thread, Client, ClientResult},
    clockwork_utils::explorer::Explorer,
    solana_sdk::{signature::read_keypair_file, transaction::Transaction},
};

fn main() -> ClientResult<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Initialize the event_stream program
    initialize(&client)?;

    // Ping a new event every 10 secs
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(10));
        ping(&client)?;
    }

    Ok(())
}

fn initialize(client: &Client) -> ClientResult<()> {
    let thread_label = "event_22-2110".to_string();
    let authority = event_stream::state::Authority::pubkey();
    let event_thread = Thread::pubkey(authority, thread_label.clone());

    // Airdrop to event thread
    client.airdrop(&event_thread, LAMPORTS_PER_SOL)?;

    let initialize_ix = Instruction {
        program_id: event_stream::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new_readonly(clockwork_client::thread::ID, false),
            AccountMeta::new(event_stream::state::Event::pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(event_thread, false),
        ],
        data: event_stream::instruction::Initialize {thread_label}.data(),
    };

    sign_send_and_confirm_tx(client, initialize_ix, "initialize".into())?;

    Ok(())
}

fn ping(client: &Client) -> ClientResult<()> {
    let ping_ix = Instruction {
        program_id: event_stream::ID,
        accounts: vec![
            AccountMeta::new(event_stream::state::Event::pubkey(), false),
            AccountMeta::new(client.payer_pubkey(), true),
        ],
        data: event_stream::instruction::Ping {}.data(),
    };
    sign_send_and_confirm_tx(client, ping_ix, "ping".into())?;

    Ok(())
}

fn sign_send_and_confirm_tx(client: &Client, ix: Instruction, label: String) -> ClientResult<()> {
    // Create tx
    let mut tx = Transaction::new_with_payer(&[ix], Some(&client.payer_pubkey()));
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
