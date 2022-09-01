use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{system_program, sysvar, instruction::Instruction},
    },
    anchor_spl::{
        associated_token::{self, get_associated_token_address}, token::{self, Mint}
    },
    clockwork_crank::{ 
        cpi::accounts::QueueUpdate,
        program::ClockworkCrank,
        state::{SEED_QUEUE, Queue},
    },
};

#[derive(Accounts)]
#[instruction(new_recipient: Option<Pubkey>)]
pub struct SetRecipient<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(address = clockwork_crank::ID)]
    pub clockwork_program: Program<'info, ClockworkCrank>,

    #[account(
        mut,
        seeds = [SEED_DISTRIBUTOR, distributor.mint.as_ref(), distributor.authority.as_ref()],
        bump,
        has_one = mint,
        has_one = authority,
    )]
    pub distributor: Account<'info, Distributor>,

    #[account(
        mut, 
        seeds = [
            SEED_QUEUE, 
            distributor.key().as_ref(), 
            "distributor".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump
     )]
    pub distributor_queue: Account<'info, Queue>,
    
    pub mint: Account<'info, Mint>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, SetRecipient<'info>>, new_recipient: Option<Pubkey>) -> Result<()> {
     // get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let distributor = &mut ctx.accounts.distributor;
    let distributor_queue = &mut ctx.accounts.distributor_queue;
    let mint = &ctx.accounts.mint;
    let system_program = &ctx.accounts.system_program;

    // get distributor bump
    let bump = *ctx.bumps.get("distributor").unwrap();

    // update distributor with new recipient
    if let Some(new_recipient) = new_recipient {
        distributor.recipient = new_recipient;
    }

    // get recipient's ATA
    let recipient_token_account_pubkey = get_associated_token_address(&distributor.recipient, &distributor.mint);

    // new ix data
    let mint_token_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(distributor.key(), false),
            AccountMeta::new(distributor_queue.key(), true),
            AccountMeta::new(mint.key(), false),
            AccountMeta::new(clockwork_crank::payer::ID, true),
            AccountMeta::new_readonly(distributor.recipient, false),
            AccountMeta::new(recipient_token_account_pubkey, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),

        ],
        data: clockwork_crank::anchor::sighash("mint_token").to_vec()
    };

    // update distributor queue
    clockwork_crank::cpi::queue_update(
    CpiContext::new_with_signer(
    clockwork_program.to_account_info(),
        QueueUpdate {
                    authority: distributor.to_account_info(), 
                    queue: distributor_queue.to_account_info(), 
                    system_program: system_program.to_account_info()
                },             
        &[&[SEED_DISTRIBUTOR, distributor.mint.as_ref(), distributor.authority.as_ref(), &[bump]]],
        ),
    Some(mint_token_ix.into()), 
    None
    )?;


    Ok(())
}
