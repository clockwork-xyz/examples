use {
    crate::{error::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = payer,
    )]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(mut, address = Subscription::pda(subscription.owner.key(),subscription.subscription_id.clone()).0)]
    pub subscription: Account<'info, Subscription>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = subscription,
        address = Subscription::bank_pda(subscription.key(),subscription.owner.key()).0
    )]
    pub subscription_bank: Account<'info, TokenAccount>,

    #[account(address=subscription.mint)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Withdraw<'_> {
    pub fn process(&mut self) -> Result<()> {
        let Self {
            payer,
            subscription,
            token_program,
            payer_token_account,
            subscription_bank,
            ..
        } = self;

        require!(subscription.owner == payer.key(), ErrorCode::NotOwner);

        token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                Transfer {
                    authority: subscription.to_account_info(),
                    from: subscription_bank.to_account_info(),
                    to: payer_token_account.to_account_info(),
                },
                &[&[
                    SEED_SUBSCRIPTION,
                    payer.key().as_ref(),
                    &subscription.subscription_id.to_be_bytes(),
                    &[subscription.bump],
                ]],
            ),
            subscription_bank.amount,
        )?;

        Ok(())
    }
}
