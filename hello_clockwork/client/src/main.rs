use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            native_token::LAMPORTS_PER_SOL,
            system_program,
        },
        InstructionData,
    },
    clockwork_sdk::{Client, ClientResult},
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
    let authority = hello_clockwork::state::Authority::pubkey();
    let hello_queue =
        clockwork_sdk::queue_program::accounts::Queue::pubkey(authority, "hello".to_string());

    // airdrop to hello queue
    client.airdrop(&hello_queue, LAMPORTS_PER_SOL)?;

    println!("{}", authority);
    println!("{}", clockwork_sdk::queue_program::ID);
    println!("{}", hello_queue);
    println!("{}", client.payer_pubkey());

    // Create ix
    let initialize_ix = Instruction {
        program_id: hello_clockwork::ID,
        accounts: vec![
            AccountMeta::new(authority, false),
            AccountMeta::new_readonly(clockwork_sdk::queue_program::ID, false),
            AccountMeta::new(hello_queue, false),
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: hello_clockwork::instruction::Initialize {}.data(),
    };

    send_and_confirm_tx(&client, initialize_ix, "initialize".to_string())?;

    println!(
        "queue: https://explorer.solana.com/address/{}?cluster=custom",
        hello_queue
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
