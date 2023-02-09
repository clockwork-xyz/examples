use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            native_token::LAMPORTS_PER_SOL,
        },
        InstructionData,
    },
    clockwork_client::{
        thread::{
            instruction::thread_create,
            ID as thread_program_ID,
            state::{Thread, Trigger},
        },
        Client, ClientResult,
    },
    clockwork_utils::{explorer::Explorer},
    solana_sdk::{signature::read_keypair_file, transaction::Transaction},
};

fn main() -> ClientResult<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    let thread_authority = client.payer_pubkey();

    // Derive PDAs:
    let thread_label = "hello";
    let hello_thread = Thread::pubkey(thread_authority, thread_label.into());

    // airdrop to hello thread
    client.airdrop(&hello_thread, LAMPORTS_PER_SOL)?;

    // Create ix
    let hello_world_ix = Instruction {
        program_id: hello_clockwork::ID,
        accounts: vec![AccountMeta::new(hello_thread, true)],
        data: hello_clockwork::instruction::HelloWorld { name: "Bob".into() }.data(),
    };

    let thread_create = thread_create(
        thread_authority,
        thread_label.into(),
        hello_world_ix.into(),
        client.payer_pubkey(),
        hello_thread,
        Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        },
    );

    send_and_confirm_tx(&client, thread_create, "thread_create".into())?;
    println!(
        "thread: 🔗 {}",
        explorer().thread_url(hello_thread, thread_program_ID)
    );

    Ok(())
}

fn send_and_confirm_tx(client: &Client, ix: Instruction, label: String) -> ClientResult<()> {
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
