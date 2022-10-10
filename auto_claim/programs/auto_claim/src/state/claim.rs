use anchor_lang::{prelude::*, AnchorDeserialize};

pub const SEED_CLAIM: &[u8] = b"claim";

/**
 * Claim
 */

#[account]
#[derive(Debug)]
pub struct Claim {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub schedule: String,
}

impl Claim {
    pub fn pubkey(sender: Pubkey, recipient: Pubkey, mint: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_CLAIM,
                sender.as_ref(),
                recipient.as_ref(),
                mint.as_ref(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Claim {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Claim::try_deserialize(&mut data.as_slice())
    }
}

/**
 * ClaimAccount
 */

pub trait ClaimAccount {
    fn new(
        &mut self,
        sender: Pubkey,
        recipient: Pubkey,
        mint: Pubkey,
        schedule: String,
    ) -> Result<()>;
}

impl ClaimAccount for Account<'_, Claim> {
    fn new(
        &mut self,
        sender: Pubkey,
        recipient: Pubkey,
        mint: Pubkey,
        schedule: String,
    ) -> Result<()> {
        self.sender = sender;
        self.recipient = recipient;
        self.mint = mint;
        self.schedule = schedule;
        Ok(())
    }
}
