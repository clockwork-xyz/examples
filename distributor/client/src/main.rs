use {
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
        InstructionData,
    },
    anchor_spl::{associated_token, token},
    clockwork_client::{
        thread::{
            instruction::thread_create,
            ID as thread_program_ID,
            state::{Thread, Trigger},
        },
        Client, ClientResult, SplToken,
    },
    clockwork_utils::{explorer::Explorer, PAYER_PUBKEY},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signature::read_keypair_file, signer::Signer, transaction::Transaction,
    },
};

fn main() -> ClientResult<()> {
    // Creating a Client with your default paper keypair as payer
    let client = default_client();
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // Security:
    // Note that we are using your default Solana paper keypair as the thread authority.
    // Feel free to use whichever authority is appropriate for your use case.
    let thread_authority = client.payer_pubkey();

    let bob = Keypair::new().pubkey();
    let charlie = Keypair::new().pubkey();

    // airdrop to alice
    client.airdrop(&client.payer_pubkey(), 2 * LAMPORTS_PER_SOL)?;

    // initialize mint
    let mint = client
        .create_token_mint(&client.payer_pubkey(), 9)
        .unwrap()
        .pubkey();

    // derive distributor program PDAs
    let distributor = distributor::state::Distributor::pubkey(mint, client.payer_pubkey());
    let distributor_thread = Thread::pubkey(
        thread_authority,
        "distributor".into(),
    );

    // get ATAs
    let bobs_token_account =
        anchor_spl::associated_token::get_associated_token_address(&bob, &mint);
    let charlies_token_account =
        anchor_spl::associated_token::get_associated_token_address(&charlie, &mint);

    print_explorer_link(bobs_token_account, "bob's token account".into())?;
    print_explorer_link(charlies_token_account, "charlie's token account".into())?;

    create_distributor(&client, distributor, mint, bob, bobs_token_account)?;

    // airdrop distributor thread
    client.airdrop(&distributor_thread, 2 * LAMPORTS_PER_SOL)?;

    let distribute_ix = Instruction {
        program_id: distributor::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(distributor, false),
            AccountMeta::new(distributor_thread.key(), true),
            AccountMeta::new(mint.key(), false),
            AccountMeta::new(PAYER_PUBKEY, true),
            AccountMeta::new_readonly(bob.key(), false),
            AccountMeta::new(bobs_token_account.key(), false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: distributor::instruction::Distribute.data(),
    };

    let thread_create = thread_create(
        thread_authority,
        "distributor".into(),
        distribute_ix.into(),
        client.payer_pubkey(),
        distributor_thread,
        Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        },
    );

    sign_send_and_confirm_tx(&client, vec![thread_create], None, "thread_create".into())?;
    println!(
        "thread: 🔗 {}",
        explorer().thread_url(distributor_thread, thread_program_ID)
    );

    // wait 10 seconds to update distributor
    println!("wait 10 seconds to update distributor");
    for n in 0..10 {
        println!("{}", n);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    update_distributor(&client, distributor, distributor_thread, mint, charlie)?;

    Ok(())
}

fn create_distributor(
    client: &Client,
    distributor: Pubkey,
    mint: Pubkey,
    bob: Pubkey,
    bobs_token_account: Pubkey,
) -> ClientResult<()> {
    let create_distributor_ix = Instruction {
        program_id: distributor::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(distributor, false),
            AccountMeta::new_readonly(bob, false),
            AccountMeta::new(bobs_token_account, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: distributor::instruction::Create {
            mint_amount: 100_000_000,
        }
        .data(),
    };

    sign_send_and_confirm_tx(
        client,
        [create_distributor_ix].to_vec(),
        None,
        "create_distributor".into(),
    )?;

    Ok(())
}

fn update_distributor(
    client: &Client,
    distributor: Pubkey,
    distributor_thread: Pubkey,
    mint: Pubkey,
    charlie: Pubkey,
) -> ClientResult<()> {
    let update_ix = Instruction {
        program_id: distributor::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(thread_program_ID, false),
            AccountMeta::new(distributor, false),
            AccountMeta::new(distributor_thread, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: distributor::instruction::Update {
            new_recipient: Some(charlie),
            mint_amount: Some(200_000_000),
            trigger: Some(Trigger::Cron {
                schedule: "*/15 * * * * * *".into(),
                skippable: true,
            }),
        }
        .data(),
    };

    sign_send_and_confirm_tx(client, vec![update_ix], None, "update_distributor".into())?;

    Ok(())
}

pub fn print_explorer_link(address: Pubkey, label: String) -> ClientResult<()> {
    println!(
        "{}: https://explorer.solana.com/address/{}?cluster=custom",
        label.to_string(),
        address
    );

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

    // Send and confirm initialize tx
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
