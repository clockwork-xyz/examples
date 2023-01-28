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
    pub authority: Pubkey,
    pub pc_mint: Pubkey,
    pub coin_mint: Pubkey,
    pub swap_amount: u64,
}

impl Investment {
    pub fn pubkey(authority: Pubkey, market: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[SEED_INVESTMENT, authority.as_ref(), market.as_ref()],
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
    fn new(
        &mut self,
        authority: Pubkey,
        market: Pubkey,
        pc_mint: Pubkey,
        coin_mint: Pubkey,
        swap_amount: u64,
    ) -> Result<()>;
}

impl InvestmentAccount for Account<'_, Investment> {
    fn new(
        &mut self,
        authority: Pubkey,
        market: Pubkey,
        pc_mint: Pubkey,
        coin_mint: Pubkey,
        swap_amount: u64,
    ) -> Result<()> {
        self.authority = authority;
        self.market = market;
        self.pc_mint = pc_mint;
        self.coin_mint = coin_mint;
        self.swap_amount = swap_amount;
        Ok(())
    }
}
