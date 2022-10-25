use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            native_token::LAMPORTS_PER_SOL,
        },
        InstructionData,
    },
    clockwork_sdk::client::{
        thread_program::{
            instruction::thread_create,
            objects::{Thread, Trigger},
        },
        Client, ClientResult,
    },
    solana_sdk::{signature::Keypair, transaction::Transaction},
};

fn main() -> ClientResult<()> {
    // Create Client
    let payer = Keypair::new();
    #[cfg(feature = "devnet")]
    let client = Client::new(payer, "https://api.devnet.solana.com".into());
    #[cfg(not(feature = "devnet"))]
    let client = Client::new(payer, "http://localhost:8899".into());

    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Derive PDAs
    let hello_thread = Thread::pubkey(client.payer_pubkey(), "hello".into());

    // airdrop to hello thread
    client.airdrop(&hello_thread, LAMPORTS_PER_SOL)?;

    // Create ix
    let hello_world_ix = Instruction {
        program_id: hello_clockwork::ID,
        accounts: vec![AccountMeta::new(hello_thread, true)],
        data: hello_clockwork::instruction::HelloWorld { name: "Bob".into() }.data(),
    };

    let thread_create = thread_create(
        client.payer_pubkey(),
        "hello".into(),
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
        "thread: https://explorer.solana.com/address/{}?cluster=custom",
        hello_thread
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
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }

    Ok(())
}
