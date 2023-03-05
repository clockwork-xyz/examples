use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        InstructionData,
        solana_program::{system_program, sysvar, instruction::Instruction},
    },
    anchor_spl::{
        associated_token::{self, get_associated_token_address}, token::{self, Mint},
    },
    clockwork_sdk::{
        ID as thread_program_ID,
        cpi::{
            ThreadUpdate
        },
        state::{
            ThreadAccount, Thread,
            Trigger, ThreadSettings,
        },
        ThreadProgram,
        utils::PAYER_PUBKEY,
    },
};

#[derive(Accounts)]
#[instruction(new_recipient: Option < Pubkey >, mint_amount: Option < u64 >, trigger: Option < String >)]
pub struct Update<'info> {
    #[account(
    mut,
    address = distributor_thread.authority
    )]
    pub authority: Signer<'info>,

    #[account(address = thread_program_ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,

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
    address = distributor_thread.pubkey(),
    constraint = distributor_thread.authority.eq(&distributor.authority),
    )]
    pub distributor_thread: Account<'info, Thread>,
    
    pub mint: Account<'info, Mint>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, Update<'info>>,
    new_recipient: Option<Pubkey>,
    mint_amount: Option<u64>,
    schedule: Option<String>,
) -> Result<()> {
    // get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let authority = &ctx.accounts.authority;
    let distributor = &mut ctx.accounts.distributor;
    let distributor_thread = &mut ctx.accounts.distributor_thread;
    let mint = &ctx.accounts.mint;
    let system_program = &ctx.accounts.system_program;

    // get distributor bump
    let bump = *ctx.bumps.get("distributor").unwrap();

    // update mint amount
    if let Some(mint_amount) = mint_amount {
        distributor.mint_amount = mint_amount;
    }

    // update new recipient
    if let Some(new_recipient) = new_recipient {
        distributor.recipient = new_recipient;
        distributor.recipient_token_account = get_associated_token_address(&new_recipient, &distributor.mint);
    }

    let mint_token_ix = Instruction {
        program_id: crate::ID,
        accounts: crate::accounts::Distribute {
            associated_token_program: associated_token::ID,
            distributor: distributor.key(),
            distributor_thread: distributor_thread.key(),
            mint: mint.key(),
            payer: PAYER_PUBKEY,
            recipient: distributor.recipient.key(),
            recipient_token_account: distributor.recipient_token_account.key(),
            rent: sysvar::rent::ID,
            system_program: system_program::ID,
            token_program: token::ID,
        }.to_account_metas(Some(true)),
        data: crate::instruction::Distribute{}.data()
    }.into();

    let mut trigger: Option<Trigger> = None;
    if let Some(schedule) = schedule {
        trigger = Some(Trigger::Cron {
            schedule,
            skippable: true,
        });
    }

    // update distributor thread
    clockwork_sdk::cpi::thread_update(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            ThreadUpdate {
                authority: authority.to_account_info(),
                thread: distributor_thread.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_DISTRIBUTOR, distributor.mint.as_ref(), distributor.authority.as_ref(), &[bump]]],
        ),
        ThreadSettings {
            instructions: Some(vec![mint_token_ix]),
            fee: None,
            name: None,
            rate_limit: None,
            trigger,
        },
    )?;


    Ok(())
}
