use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_CRANK: &[u8] = b"crank";

/**
 * Crank
 */

#[account]
#[derive(Debug)]
pub struct Crank {
    pub authority: Pubkey,
    pub market: Pubkey,
    pub limit: u16,
}

impl Crank {
    pub fn pubkey(authority: Pubkey, market: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[SEED_CRANK, authority.as_ref(), market.as_ref()],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Crank {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Crank::try_deserialize(&mut data.as_slice())
    }
}

/**
 * CrankAccount
 */

pub trait CrankAccount {
    fn new(&mut self, authority: Pubkey, market: Pubkey, limit: u16) -> Result<()>;
}

impl CrankAccount for Account<'_, Crank> {
    fn new(&mut self, authority: Pubkey, market: Pubkey, limit: u16) -> Result<()> {
        self.authority = authority;
        self.market = market;
        self.limit = limit;
        Ok(())
    }
}
