use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_SUBSCRIBER: &[u8] = b"subscriber";

/**
 * Subscriber
 */

#[account]
#[derive(Debug)]
pub struct Subscriber {
    pub owner: Pubkey,
    pub subscription: Pubkey,
    pub is_active: bool,
    pub last_transfer_at: Option<i64>,
    pub bump: u8,
}

impl Subscriber {
    pub fn pda(owner: Pubkey, subscription: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[SEED_SUBSCRIBER, owner.as_ref(), subscription.as_ref()],
            &crate::ID,
        )
    }
}

impl TryFrom<Vec<u8>> for Subscriber {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Subscriber::try_deserialize(&mut data.as_slice())
    }
}

pub trait SubscriberAccount {
    fn new(&mut self, owner: Pubkey, subscription: Pubkey, is_active: bool, bump: u8)
        -> Result<()>;
}

impl SubscriberAccount for Account<'_, Subscriber> {
    fn new(
        &mut self,
        owner: Pubkey,
        subscription: Pubkey,
        is_active: bool,
        bump: u8,
    ) -> Result<()> {
        self.owner = owner;
        self.subscription = subscription;
        self.is_active = is_active;
        self.last_transfer_at = None;
        self.bump = bump;
        Ok(())
    }
}
