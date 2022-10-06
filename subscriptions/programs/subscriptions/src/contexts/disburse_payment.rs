use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{ 
        associated_token::AssociatedToken,
        token::{self,TokenAccount,Transfer}
    },
    clockwork_crank::state::{Queue, SEED_QUEUE, CrankResponse},
};

#[derive(Accounts)]
pub struct DisbursePayment<'info> {
    pub subscriber: Signer<'info>,
    #[account(
        mut,
        associated_token::authority = subscriber,
        associated_token::mint = subscription.mint,
    )]
    pub subscriber_token_account: Account<'info,TokenAccount>,
    #[account(address = subscription.owner)]
    pub owner: AccountInfo<'info>,
    #[account( 
        address = subscription.subscription_bank
    )]
    pub subscription_bank: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = Subscription::pubkey(subscription.owner,subscription.market_id.clone())
    )]
    pub subscription: Box<Account<'info, Subscription>>,
    #[account(
        signer, 
        seeds = [
            SEED_QUEUE, 
            subscription.key().as_ref(), 
            "subscription".as_bytes()
        ], 
        seeds::program = clockwork_crank::ID,
        bump,
    )]
    pub subscription_queue: Box<Account<'info, Queue>>,

    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,

}

impl<'info> DisbursePayment<'_> {
    pub fn process(
        &mut self,
    ) -> Result<CrankResponse> {
        let Self {
            owner,
            subscriber,
            subscriber_token_account,
            subscription,
            subscription_bank,
            token_program,
            ..
        } = self;

    token::transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: subscriber_token_account.to_account_info(),
                to: subscription_bank.to_account_info(),
                authority: subscriber.to_account_info(),
            },             
    ),subscription.recurrent_amount)?;

    Ok(CrankResponse{ next_instruction: None })
    }
}


