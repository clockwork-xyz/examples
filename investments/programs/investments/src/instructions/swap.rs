use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        __private::bytemuck::Contiguous,
        solana_program::{system_program, sysvar, instruction::Instruction},
    },
    anchor_spl::{
        dex::{
            serum_dex::{
                instruction::SelfTradeBehavior,
                matching::{OrderType, Side},
            },
            NewOrderV3,
        },
        token::{Token, TokenAccount},
    },
    std::num::NonZeroU64,
    clockwork_sdk::state::{Thread, ThreadAccount, ThreadResponse},
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(address = anchor_spl::dex::ID)]
    pub dex_program: Program<'info, OpenBookDex>,

    #[account(
        seeds = [
            SEED_INVESTMENT, 
            investment.authority.key().as_ref(), 
            investment.market.key().as_ref(), 
        ], 
        bump,
    )]
    pub investment: Account<'info, Investment>,

    #[account(
        signer,
        address = investment_thread.pubkey(),
        constraint = investment_thread.authority == investment.authority
    )]
    pub investment_thread: Account<'info, Thread>,

    #[account(
        mut,
        associated_token::authority = investment,
        associated_token::mint = investment.pc_mint,
    )]
    pub investment_pc_vault: Account<'info, TokenAccount>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Swap<'info>>) -> Result<ThreadResponse> {
    // get accounts
    let dex_program = &ctx.accounts.dex_program;
    let investment = &ctx.accounts.investment;
    let investment_thread = &ctx.accounts.investment_thread;
    let investment_pc_vault= &mut ctx.accounts.investment_pc_vault;
    let rent = &ctx.accounts.rent;
    let token_program = &ctx.accounts.token_program;

    // get remaining accounts
    let market = &mut ctx.remaining_accounts.get(0).unwrap();
    let event_queue = &mut ctx.remaining_accounts.get(1).unwrap();
    let request_queue = &mut ctx.remaining_accounts.get(2).unwrap();
    let market_bids = &mut ctx.remaining_accounts.get(3).unwrap();
    let market_asks = &mut ctx.remaining_accounts.get(4).unwrap();
    let coin_vault = &mut ctx.remaining_accounts.get(5).unwrap();
    let pc_vault = &mut ctx.remaining_accounts.get(6).unwrap();
    let open_orders = &mut ctx.remaining_accounts.get(7).unwrap();
    
    // get investment bump
    let bump = *ctx.bumps.get("investment").unwrap();

    // place order on serum dex
    anchor_spl::dex::new_order_v3(
        CpiContext::new_with_signer(
            dex_program.to_account_info(),
            NewOrderV3 {
                market: market.to_account_info(),
                coin_vault: coin_vault.to_account_info(),
                pc_vault: pc_vault.to_account_info(),
                request_queue: request_queue.to_account_info(),
                event_queue: event_queue.to_account_info(),
                market_bids: market_bids.to_account_info(),
                market_asks: market_asks.to_account_info(),
                open_orders: open_orders.to_account_info(),
                order_payer_token_account: investment_pc_vault.to_account_info(),
                open_orders_authority: investment.to_account_info(),
                token_program: token_program.to_account_info(),
                rent: rent.to_account_info(),
            },
            &[&[
                SEED_INVESTMENT,
                investment.authority.as_ref(),
                investment.market.as_ref(),
                &[bump],
            ]],
        ),
        Side::Bid,
        NonZeroU64::new(NonZeroU64::MAX_VALUE).unwrap(),
        NonZeroU64::new(NonZeroU64::MAX_VALUE).unwrap(),
        NonZeroU64::new(investment.swap_amount).unwrap(),
        SelfTradeBehavior::DecrementTake,
        OrderType::Limit,
        019269,
        std::u16::MAX,
    )?;

    let authority_pc_vault_pubkey = 
        anchor_spl::associated_token::get_associated_token_address(
            &investment.authority,
            &investment.pc_mint,
    );

    Ok(ThreadResponse {
        kickoff_instruction: Some(
            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(authority_pc_vault_pubkey, false),
                    AccountMeta::new_readonly(investment.key(), false),
                    AccountMeta::new(investment_pc_vault.key(), false),
                    AccountMeta::new_readonly(investment_thread.key(), true),
                    AccountMeta::new_readonly(market.key(), false),
                    AccountMeta::new_readonly(system_program::ID, false),
                    AccountMeta::new_readonly(token_program.key(), false),
                    // REMAINING ACCOUNTS
                    AccountMeta::new(event_queue.key(), false),
                    AccountMeta::new(request_queue.key(), false),
                    AccountMeta::new(market_bids.key(), false),
                    AccountMeta::new(market_asks.key(), false),
                    AccountMeta::new(coin_vault.key(), false),
                    AccountMeta::new(pc_vault.key(), false),
                    AccountMeta::new(open_orders.key(), false),
                ],
                data: clockwork_sdk::utils::anchor_sighash("deposit").into(),
            }
            .into(),
        ),
        next_instruction: None,
    })
}

    // // validation for which vault is the pc/coin vault
    // let mut pc_vault = mint_a_vault.as_ref();
    // let mut coin_vault = mint_b_vault.as_ref();

    // if let Some(mint_a_vault_mint) = 
    //     <SplTokenAccount as GenericTokenAccount>::unpack_account_mint(
    //         &mint_a_vault.try_borrow_data().unwrap()
    //     ){
    //         if mint_a_vault_mint.ne(&investment.coin_mint) {
    //             pc_vault = mint_b_vault;
    //             coin_vault = mint_a_vault;
    //         }   
    //     };