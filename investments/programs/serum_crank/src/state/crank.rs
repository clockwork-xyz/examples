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
    pub open_orders: Vec<Pubkey>,
}

impl Crank {
    pub fn pubkey() -> Pubkey {
        Pubkey::find_program_address(&[SEED_CRANK], &crate::ID).0
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
    fn new(&mut self) -> Result<()>;
    fn index(&mut self, open_orders: Vec<Pubkey>) -> Result<()>;
}

impl CrankAccount for Account<'_, Crank> {
    fn new(&mut self) -> Result<()> {
        self.open_orders = Vec::new();
        Ok(())
    }

    fn index(&mut self, open_orders: Vec<Pubkey>) -> Result<()> {
        self.open_orders = open_orders;
        Ok(())
    }
}
