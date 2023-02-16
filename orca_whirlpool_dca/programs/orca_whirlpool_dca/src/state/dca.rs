use {anchor_lang::prelude::*, clockwork_macros::TryFromData, std::convert::TryFrom};

pub const SEED_DCA: &[u8] = b"dca";

/**
 * Dca
 */

#[account]
#[derive(Debug, TryFromData)]
pub struct Dca {
    pub authority: Pubkey,
    pub whirlpool: Pubkey,
    pub a_mint: Pubkey,
    pub b_mint: Pubkey,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
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

/**
 * DCAAccount
 */

pub trait DcaAccount {
    fn new(
        &mut self,
        authority: Pubkey,
        whirlpool: Pubkey,
        a_mint: Pubkey,
        b_mint: Pubkey,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()>;
}

impl DcaAccount for Account<'_, Dca> {
    fn new(
        &mut self,
        authority: Pubkey,
        whirlpool: Pubkey,
        a_mint: Pubkey,
        b_mint: Pubkey,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        self.authority = authority;
        self.whirlpool = whirlpool;
        self.a_mint = a_mint;
        self.b_mint = b_mint;
        self.amount = amount;
        self.other_amount_threshold = other_amount_threshold;
        self.sqrt_price_limit = sqrt_price_limit;
        self.amount_specified_is_input = amount_specified_is_input;
        self.a_to_b = a_to_b;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DcaSettings {
    pub amount: Option<u64>,
    pub other_amount_threshold: Option<u64>,
    pub sqrt_price_limit: Option<u128>,
    pub amount_specified_is_input: Option<bool>,
    pub a_to_b: Option<bool>,
}
