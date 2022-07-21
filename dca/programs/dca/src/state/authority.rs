use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_AUTHORITY: &[u8] = b"authority";

/**
 * Authority
 */

#[account]
#[derive(Debug)]
pub struct Authority {
    pub manager: Pubkey,
    pub payer: Pubkey,
}

impl Authority {
    pub fn pubkey(payer: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_AUTHORITY, payer.as_ref()], &crate::ID).0
    }
}

impl TryFrom<Vec<u8>> for Authority {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Authority::try_deserialize(&mut data.as_slice())
    }
}

/**
 * AuthorityAccount
 */

pub trait AuthorityAccount {
    fn new(&mut self, manager: Pubkey, payer: Pubkey) -> Result<()>;
}

impl AuthorityAccount for Account<'_, Authority> {
    fn new(&mut self, manager: Pubkey, payer: Pubkey) -> Result<()> {
        self.manager = manager;
        self.payer = payer;
        Ok(())
    }
}
