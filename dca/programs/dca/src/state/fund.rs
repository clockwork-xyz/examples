use {
    crate::pda::PDA,
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_FUND: &[u8] = b"fund";

/**
 * Fund
 */

#[account]
#[derive(Debug)]
pub struct Fund {
    pub manager: Pubkey,
}

impl Fund {
    pub fn pda(manager: Pubkey) -> PDA {
        Pubkey::find_program_address(&[SEED_FUND, manager.as_ref()], &crate::ID)
    }
}

impl TryFrom<Vec<u8>> for Fund {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Fund::try_deserialize(&mut data.as_slice())
    }
}

pub trait FundAccount {
    fn new(&mut self, manager: Pubkey) -> Result<()>;
}

impl FundAccount for Account<'_, Fund> {
    fn new(&mut self, manager: Pubkey) -> Result<()> {
        self.manager = manager;
        Ok(())
    }
}
