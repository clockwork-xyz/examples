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
    pub market: Pubkey,
    pub event_queue: Pubkey,
    pub mint_a_vault: Pubkey,
    pub mint_b_vault: Pubkey,
    pub limit: u16,
}

impl Crank {
    pub fn pubkey(market: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_CRANK, market.as_ref()], &crate::ID).0
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
    fn new(
        &mut self,
        market: Pubkey,
        event_queue: Pubkey,
        mint_a_vault: Pubkey,
        mint_b_vault: Pubkey,
        limit: u16,
    ) -> Result<()>;
}

impl CrankAccount for Account<'_, Crank> {
    fn new(
        &mut self,
        market: Pubkey,
        event_queue: Pubkey,
        mint_a_vault: Pubkey,
        mint_b_vault: Pubkey,
        limit: u16,
    ) -> Result<()> {
        self.open_orders = Vec::new();
        self.market = market;
        self.event_queue = event_queue;
        self.mint_a_vault = mint_a_vault;
        self.mint_b_vault = mint_b_vault;
        self.limit = limit;
        Ok(())
    }
}
