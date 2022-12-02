use {
    crate::state::*,
    anchor_lang::{prelude::*, solana_program::sysvar},
    anchor_spl::token::{Mint, TokenAccount},
    std::mem::size_of,
};

#[derive(Accounts)]
#[instruction(recurrent_amount:u64,schedule:String,mint:Pubkey,is_active:bool,subscription_id: u64)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = subscription,
        seeds=[
            subscription.key().as_ref(),
            owner.key().as_ref(),
            "subscription_bank".as_bytes(),
        ],
        bump,
    )]
    pub subscription_bank: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        space = 8 + size_of::<Subscription>(),
        seeds=[
            SEED_SUBSCRIPTION,
            owner.key().as_ref(),
            &subscription_id.to_be_bytes()
        ],
        bump,
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
        subscription_id: u64,
        bump: u8,
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
            bump,
        )?;

        Ok(())
    }
}
