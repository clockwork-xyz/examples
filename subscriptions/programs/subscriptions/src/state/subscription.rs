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
    pub mint: Pubkey,
    pub recurrent_amount: u64,
    pub schedule: String,
    pub is_active: bool,
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

    pub fn bank_pubkey(subscription: Pubkey, owner: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                subscription.as_ref(),
                owner.as_ref(),
                "subscription_bank".as_bytes(),
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
        mint: Pubkey,
        recurrent_amount: u64,
        schedule: String,
        is_active: bool,
        subscription_id: String,
    ) -> Result<()>;
}

impl SubscriptionAccount for Account<'_, Subscription> {
    fn new(
        &mut self,
        owner: Pubkey,
        mint: Pubkey,
        recurrent_amount: u64,
        schedule: String,
        is_acitve: bool,
        subscription_id: String,
    ) -> Result<()> {
        self.owner = owner;
        self.mint = mint;
        self.recurrent_amount = recurrent_amount;
        self.schedule = schedule;
        self.is_active = is_acitve;
        self.subscription_id = subscription_id;
        Ok(())
    }
}
