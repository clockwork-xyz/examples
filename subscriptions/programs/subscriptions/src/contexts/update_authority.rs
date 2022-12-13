use {crate::state::*, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0, has_one=owner)]
    pub subscription: Account<'info, Subscription>,
}

impl<'info> UpdateAuthority<'_> {
    pub fn process(&mut self, new_authority: Pubkey) -> Result<()> {
        let Self { subscription, .. } = self;

        subscription.owner = new_authority;

        Ok(())
    }
}
