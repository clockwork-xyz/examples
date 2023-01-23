use {
    anchor_lang::{prelude::*, AnchorDeserialize},
    std::convert::TryFrom,
};

pub const SEED_INVESTMENT: &[u8] = b"investment";

/**
 * Investment
 */

#[account]
#[derive(Debug)]
pub struct Investment {
    pub market: Pubkey,
    pub payer: Pubkey,
    pub swap_amount: u64,
}

impl Investment {
    pub fn pubkey(payer: Pubkey, market: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[SEED_INVESTMENT, payer.as_ref(), market.as_ref()],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Investment {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Investment::try_deserialize(&mut data.as_slice())
    }
}

/**
 * InvestmentAccount
 */

pub trait InvestmentAccount {
    fn new(&mut self, payer: Pubkey, swap_amount: u64, market: Pubkey) -> Result<()>;
}

impl InvestmentAccount for Account<'_, Investment> {
    fn new(&mut self, payer: Pubkey, swap_amount: u64, market: Pubkey) -> Result<()> {
        self.payer = payer;
        self.swap_amount = swap_amount;
        self.market = market;
        Ok(())
    }
}
