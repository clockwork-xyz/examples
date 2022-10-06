use {
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar},
        InstructionData,
    },
    anchor_spl::{associated_token, token},
    clockwork_sdk::client::{queue_program, Client, ClientResult, SplToken},
    solana_sdk::{
        instruction::Instruction, native_token::LAMPORTS_PER_SOL, signature::Keypair,
        signer::Signer, transaction::Transaction,
    },
};

fn main() -> ClientResult<()> {
    // Create Client
    let payer = Keypair::new();
    #[cfg(feature = "devnet")]
    let client = Client::new(payer, "https://api.devnet.solana.com".into());
    #[cfg(not(feature = "devnet"))]
    let client = Client::new(payer, "http://localhost:8899".into());

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
    let distributor_queue =
        clockwork_sdk::queue_program::accounts::Queue::pubkey(distributor, "distributor".into());

    print_explorer_link(distributor, "distributor".into())?;
    print_explorer_link(distributor_queue, "distributor_queue".into())?;

    // get ATAs
    let bobs_token_account =
        anchor_spl::associated_token::get_associated_token_address(&bob, &mint);
    let charlies_token_account =
        anchor_spl::associated_token::get_associated_token_address(&charlie, &mint);

    print_explorer_link(bobs_token_account, "bob's token account".into())?;
    print_explorer_link(charlies_token_account, "charlie's token account".into())?;

    initialize(
        &client,
        distributor,
        distributor_queue,
        mint,
        bob,
        bobs_token_account,
    )?;

    set_recipient(&client, distributor, distributor_queue, mint, charlie)?;

    Ok(())
}

fn initialize(
    client: &Client,
    distributor: Pubkey,
    distributor_queue: Pubkey,
    mint: Pubkey,
    bob: Pubkey,
    bobs_token_account: Pubkey,
) -> ClientResult<()> {
    // airdrop to distributor queue
    client.airdrop(&distributor_queue, 2 * LAMPORTS_PER_SOL)?;

    let initialize_ix = Instruction {
        program_id: distributor::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(queue_program::ID, false),
            AccountMeta::new(mint, false),
            AccountMeta::new(distributor, false),
            AccountMeta::new(distributor_queue, false),
            AccountMeta::new_readonly(bob, false),
            AccountMeta::new(bobs_token_account, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
        ],
        data: distributor::instruction::Initialize {
            mint_amount: 100_000_000,
        }
        .data(),
    };

    sign_send_and_confirm_tx(client, [initialize_ix].to_vec(), None, "initialize".into())?;

    Ok(())
}

fn set_recipient(
    client: &Client,
    distributor: Pubkey,
    distributor_queue: Pubkey,
    mint: Pubkey,
    charlie: Pubkey,
) -> ClientResult<()> {
    let set_recipient_ix = Instruction {
        program_id: distributor::ID,
        accounts: vec![
            AccountMeta::new(client.payer_pubkey(), true),
            AccountMeta::new_readonly(queue_program::ID, false),
            AccountMeta::new(distributor, false),
            AccountMeta::new(distributor_queue, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: distributor::instruction::SetRecipient {
            new_recipient: Some(charlie),
        }
        .data(),
    };

    sign_send_and_confirm_tx(
        client,
        [set_recipient_ix].to_vec(),
        None,
        "set_recipient".into(),
    )?;

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
            "{} tx: ✅ https://explorer.solana.com/tx/{}?cluster=custom",
            label, sig
        ),
        Err(err) => println!("{} tx: ❌ {:#?}", label, err),
    }
    Ok(())
}
