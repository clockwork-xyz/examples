use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{
            system_program, sysvar, instruction::Instruction
        },
    },
    anchor_spl::{token::{self, Mint, TokenAccount},associated_token::{self,AssociatedToken}},
    clockwork_sdk::thread_program::{self, ThreadProgram, accounts::{Thread, Trigger}},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(swap_amount: u64)]
pub struct CreateInvestment<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = thread_program::ID)]
    pub clockwork_program: Program<'info, ThreadProgram>,

    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, anchor_spl::dex::Dex>,

    #[account(
        init,
        seeds = [SEED_INVESTMENT, payer.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
        payer = payer,
        space = 8 + size_of::<Investment>(),
    )]
    pub investment: Box<Account<'info, Investment>>,
    
    #[account(
        init,
        payer = payer, 
        associated_token::authority = investment, 
        associated_token::mint = mint_a
    )]
    pub investment_mint_a_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer, 
        associated_token::authority = investment, 
        associated_token::mint = mint_b
    )]
    pub investment_mint_b_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = Thread::pubkey(investment.key(), "investment".into()))]
    pub investment_thread: SystemAccount<'info>,
    
    #[account()]
    pub mint_a: Box<Account<'info, Mint>>,

    #[account()]
    pub mint_b: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        associated_token::authority = payer,
        associated_token::mint = mint_a,
    )]
    pub payer_mint_a_token_account: Box<Account<'info, TokenAccount>>,
    
    #[account(
        init,
        payer = payer,
        associated_token::authority = payer,
        associated_token::mint = mint_b,
    )]
    pub payer_mint_b_token_account: Box<Account<'info, TokenAccount>>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, CreateInvestment<'info>>, swap_amount: u64) -> Result<()> {
    // Get accounts
    let clockwork_program = &ctx.accounts.clockwork_program;
    let dex_program = &ctx.accounts.dex_program;
    let investment = &mut ctx.accounts.investment;
    let investment_mint_a_token_account = &ctx.accounts.investment_mint_a_token_account;
    let mint_a = &ctx.accounts.mint_a;
    let mint_b = &ctx.accounts.mint_b;
    let payer = &ctx.accounts.payer;
    let investment_thread = &mut ctx.accounts.investment_thread;
    let system_program = &ctx.accounts.system_program;

    // Get remaining accounts
    let market = ctx.remaining_accounts.get(0).unwrap();
    let mint_a_vault = ctx.remaining_accounts.get(1).unwrap();
    let mint_b_vault = ctx.remaining_accounts.get(2).unwrap();
    let request_thread = ctx.remaining_accounts.get(3).unwrap();
    let event_thread = ctx.remaining_accounts.get(4).unwrap();
    let market_bids = ctx.remaining_accounts.get(5).unwrap();
    let market_asks = ctx.remaining_accounts.get(6).unwrap();
    let open_orders = ctx.remaining_accounts.get(7).unwrap();

    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // initialize investment account
    investment.new(
    payer.key(), 
    mint_a.key(), 
    mint_b.key(),
    swap_amount
    )?;

    // create swap ix
    let swap_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(associated_token::ID, false),
            AccountMeta::new_readonly(dex_program.key(), false),
            AccountMeta::new_readonly(investment.key(), false),
            AccountMeta::new(investment_mint_a_token_account.key(), false),
            AccountMeta::new_readonly(investment_thread.key(), false),
            AccountMeta::new(clockwork_sdk::PAYER_PUBKEY, true),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(token::ID, false),
            // Extra Accounts
            AccountMeta::new(market.key(), false),
            AccountMeta::new(mint_a_vault.key(), false),
            AccountMeta::new(mint_b_vault.key(), false),
            AccountMeta::new(request_thread.key(), false),
            AccountMeta::new(event_thread.key(), false),
            AccountMeta::new(market_bids.key(), false),
            AccountMeta::new(market_asks.key(), false),
            AccountMeta::new(open_orders.key(), false),
        ],
        data: clockwork_sdk::anchor_sighash("swap").into(),
    };

    // Create thread
    clockwork_sdk::thread_program::cpi::thread_create(
        CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::thread_program::cpi::accounts::ThreadCreate {
                authority: investment.to_account_info(),
                payer: payer.to_account_info(),
                thread: investment_thread.to_account_info(),
                system_program: system_program.to_account_info(),
            },
            &[&[SEED_INVESTMENT, investment.payer.as_ref(), investment.mint_a.as_ref(), investment.mint_b.as_ref(), &[bump]]],
        ),
        "investment".into(),
        swap_ix.into(),
        Trigger::Cron { 
            schedule: "*/15 * * * * * *".into(),
            skippable: true
        }    
    )?;

    Ok(())
}
