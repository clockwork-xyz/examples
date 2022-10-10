use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::{system_program, sysvar, instruction::Instruction}},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, TokenAccount},
    },
    clockwork_sdk::queue_program::{self, accounts::{Queue, Trigger}, QueueProgram},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(schedule: String)]
pub struct Create<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = Queue::pubkey(claim.key(), "auto_claim".into()))]
    pub auto_claim_queue: SystemAccount<'info>,

    #[account(
        init,
        payer = sender,
        seeds = [
            SEED_CLAIM, 
            sender.key().as_ref(), 
            recipient.key().as_ref(), 
            mint.key().as_ref()
        ],
        bump,
        space = 8 + size_of::<Claim>(),
    )]
    pub claim: Account<'info, Claim>,

    pub mint: Account<'info, Mint>,

    #[account(address = queue_program::ID)]
    pub queue_program: Program<'info, QueueProgram>,

    #[account(mut)]
    pub recipient: Signer<'info>,

    #[account(
        associated_token::authority = recipient,
        associated_token::mint = mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub sender: Signer<'info>,

    #[account(
        associated_token::authority = sender,
        associated_token::mint = mint,
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = streamflow_sdk::ID)]
    pub timelock_program: Program<'info, streamflow_sdk::program::StreamflowSdk>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Create<'info>>, schedule: String) -> Result<()> {
    // get accounts
    let associated_token_program = &ctx.accounts.associated_token_program;
    let auto_claim_queue = &ctx.accounts.auto_claim_queue;
    let claim = &mut ctx.accounts.claim;
    let mint = &ctx.accounts.mint;
    let queue_program = &ctx.accounts.queue_program;
    let recipient = &ctx.accounts.recipient;
    let recipient_token_account = &ctx.accounts.recipient_token_account;
    let rent = &ctx.accounts.rent;
    let sender = &ctx.accounts.sender;
    let sender_token_account = &ctx.accounts.sender_token_account;
    let system_program = &ctx.accounts.system_program;
    let timelock_program = &ctx.accounts.timelock_program;
    let token_program = &ctx.accounts.token_program;

    // get claim bump
    let bump = *ctx.bumps.get("claim").unwrap();

    // initialize claim account
    claim.new(sender.key(), recipient.key(), mint.key(), schedule)?;

    //TODO: create vesting contract
    streamflow_sdk::cpi::create(
    CpiContext::new(
            timelock_program.to_account_info(), 
            streamflow_sdk::cpi::accounts::Create { 
                    sender:sender.to_account_info(), 
                    sender_tokens: sender_token_account.to_account_info(), 
                    recipient: recipient.to_account_info(), 
                    metadata: rent.to_account_info(), 
                    // escrow_tokens: rent.to_account_info(), 
                    recipient_tokens: recipient_token_account.to_account_info(), 
                    // streamflow_treasury: rent.to_account_info(), 
                    // streamflow_treasury_tokens: rent.to_account_info(), 
                    // withdrawor: rent.to_account_info(), 
                    // partner: rent.to_account_info(), 
                    // partner_tokens: rent.to_account_info(), 
                    mint: mint.to_account_info(), 
                    // fee_oracle: rent.to_account_info(), 
                    rent: rent.to_account_info(), 
                    timelock_program: timelock_program.to_account_info(), 
                    token_program: token_program.to_account_info(), 
                    associated_token_program: associated_token_program.to_account_info(),
                    system_program: system_program.to_account_info()
                }), 
                Clock::get().unwrap().unix_timestamp as u64, 
                100000000, 
                Clock::get().unwrap().unix_timestamp as u64 + 10000, 
                100000, 
                Clock::get().unwrap().unix_timestamp as u64, 
                100000, 
                true, 
                true, 
                false, 
                true, 
                true, 
                true, 
                "auto_claim".as_bytes().try_into().unwrap(), 
                100000
    )?;

    // define auto_claim ix
    let auto_claim_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![],
        data: clockwork_sdk::queue_program::utils::anchor_sighash("auto_claim").into(),
    };

    // initialize auto_claim_queue
    clockwork_sdk::queue_program::cpi::queue_create(
        CpiContext::new_with_signer(
            queue_program.to_account_info(),
            clockwork_sdk::queue_program::cpi::accounts::QueueCreate {
                authority: claim.to_account_info(),
                payer: sender.to_account_info(),
                queue: auto_claim_queue.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[
                SEED_CLAIM,
                claim.sender.as_ref(),
                claim.recipient.as_ref(),
                claim.mint.as_ref(),
                &[bump],
            ]],
        ),
        "auto_claim".into(),
        auto_claim_ix.into(),
        Trigger::Cron {
            schedule: claim.schedule.to_string(),
            skippable: false,
        },
    )?;


    Ok(())
}
