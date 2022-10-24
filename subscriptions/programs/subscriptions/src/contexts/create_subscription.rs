use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::sysvar},
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, TokenAccount},
    },
    clockwork_sdk::queue_program::{self, accounts::Queue, QueueProgram},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(subscription_id: String)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = subscription,
        address = Subscription::bank_pubkey(subscription.key(),owner.key())
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    #[account(
        init,
        address = Subscription::pubkey(owner.key(),subscription_id),
        payer = owner,
        space = 8 + size_of::<Subscription>(),
    )]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, anchor_spl::token::Token>,
    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> CreateSubscription<'_> {
    pub fn process(
        &mut self,
        recurrent_amount: u64,
        schedule: String,
        mint: Pubkey,
        is_active: bool,
        subscription_id: String,
    ) -> Result<()> {
        let Self {
            owner,
            subscription,
            ..
        } = self;

        subscription.new(
            owner.key(),
            mint,
            recurrent_amount,
            schedule,
            is_active,
            subscription_id,
            0,
        )?;

        Ok(())
    }
}
