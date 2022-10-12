use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_SUBSCRIPTION: &[u8] = b"subscription";

/**
 * Subscription
 */

#[account]
#[derive(Debug)]
pub struct Subscription {
    pub owner: Pubkey,
    pub subscription_bank: Pubkey,
    pub mint: Pubkey,
    pub recurrent_amount: u64,
    pub epochs_reset: u64,
    pub is_active: bool,
    pub subscribers: Vec<Pubkey>,
    pub subscription_id: String,
}

impl Subscription {
    pub fn pubkey(owner: Pubkey, subscription_id: String) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_SUBSCRIPTION,
                owner.as_ref(),
                subscription_id.as_bytes(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Subscription {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Subscription::try_deserialize(&mut data.as_slice())
    }
}

pub trait SubscriptionAccount {
    fn new(
        &mut self,
        owner: Pubkey,
        owner_token_account: Pubkey,
        mint: Pubkey,
        recurrent_amount: u64,
        epochs_reset: u64,
        is_active: bool,
        subscribers: Vec<Pubkey>,
        subscription_id: String,
    ) -> Result<()>;
}

impl SubscriptionAccount for Account<'_, Subscription> {
    fn new(
        &mut self,
        owner: Pubkey,
        subscription_bank: Pubkey,
        mint: Pubkey,
        recurrent_amount: u64,
        epochs_reset: u64,
        is_acitve: bool,
        subscribers: Vec<Pubkey>,
        subscription_id: String,
    ) -> Result<()> {
        self.owner = owner;
        self.subscription_bank = subscription_bank;
        self.mint = mint;
        self.recurrent_amount = recurrent_amount;
        self.epochs_reset = epochs_reset;
        self.is_active = is_acitve;
        self.subscribers = vec![];
        self.subscription_id = subscription_id;
        Ok(())
    }
}
