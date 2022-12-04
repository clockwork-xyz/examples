use {crate::state::*, anchor_lang::prelude::*, anchor_spl::token::Mint};

#[derive(Accounts)]
pub struct DeactivateSubscription<'info> {
    #[account(mut, address=subscription.owner)]
    pub payer: Signer<'info>,
    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(address=subscription.mint)]
    pub mint: Account<'info, Mint>,
}

impl<'info> DeactivateSubscription<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self { subscription, .. } = self;

        subscription.is_active = false;
        Ok(())
    }
}
