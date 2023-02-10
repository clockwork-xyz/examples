use {anchor_lang::prelude::*, std::convert::TryFrom};

pub const SEED_DCA: &[u8] = b"dca";

/**
 * Dca
 */

#[account]
#[derive(Debug)]
pub struct Dca {
    pub authority: Pubkey,
    pub a_mint: Pubkey,
    pub b_mint: Pubkey,
    /// SOURCE amount to transfer, output to DESTINATION is based on the exchange rate
    pub amount_in: u64,
    /// Minimum amount of DESTINATION token to output, prevents excessive slippage
    pub minimum_amount_out: u64,
}

impl Dca {
    pub fn pubkey(authority: Pubkey, a_mint: Pubkey, b_mint: Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                SEED_DCA,
                authority.as_ref(),
                a_mint.as_ref(),
                b_mint.as_ref(),
            ],
            &crate::ID,
        )
        .0
    }
}

impl TryFrom<Vec<u8>> for Dca {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Dca::try_deserialize(&mut data.as_slice())
    }
}

/**
 * DCAAccount
 */

pub trait DcaAccount {
    fn new(
        &mut self,
        authority: Pubkey,
        a_mint: Pubkey,
        b_mint: Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()>;
}

impl DcaAccount for Account<'_, Dca> {
    fn new(
        &mut self,
        authority: Pubkey,
        a_mint: Pubkey,
        b_mint: Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        self.authority = authority;
        self.a_mint = a_mint;
        self.b_mint = b_mint;
        self.amount_in = amount_in;
        self.minimum_amount_out = minimum_amount_out;
        Ok(())
    }
}
