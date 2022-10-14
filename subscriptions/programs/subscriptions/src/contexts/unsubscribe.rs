use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Unsubscribe<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = Subscriber::pubkey(payer.key(),subscription.key()),
    )]
    pub subscriber: Account<'info, Subscriber>,
    #[account(
        mut,
        associated_token::authority = payer,
        associated_token::mint = subscription.mint,
    )]
    pub subscriber_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address=Subscription::bank_pubkey(subscription.key(),subscription.owner.key())
    )]
    pub subscription_bank: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = Subscription::pubkey(subscription.owner.key(),subscription.subscription_id))]
    pub subscription: Account<'info, Subscription>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Unsubscribe<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscriber,
            subscriber_token_account,
            subscription_bank,
            subscription,
            system_program,
            token_program,
            ..
        } = self;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    from: subscription_bank.to_account_info(),
                    to: subscriber_token_account.to_account_info(),
                    authority: subscription.to_account_info(),
                },
                &[&[
                    subscription.key().as_ref(),
                    subscription.owner.as_ref(),
                    "subscription_bank".as_bytes(),
                ]],
            ),
            subscriber.locked_amount,
        )?;

        subscriber.locked_amount = 0;

        Ok(())
    }
}
