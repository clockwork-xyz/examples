use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::Mint,
};

#[derive(Accounts)]
pub struct DeactivateSubscription<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(address=subscription.mint)]
    pub mint: Account<'info, Mint>,
}

impl<'info> DeactivateSubscription<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscription,
            ..
        } = self;

        require!(subscription.owner == payer.key(), ErrorCode::NotOwner);
        subscription.is_active = false;
        Ok(())
    }
}
