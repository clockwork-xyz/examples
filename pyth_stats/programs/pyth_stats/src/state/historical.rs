use anchor_lang::prelude::*;

pub const SEED_HISTORICAL_AVGS: &[u8] = b"historical_avgs";

/**
 * HistoricalAvgs
 */

#[account(zero_copy)]
pub struct HistoricalAvgs {}

impl HistoricalAvgs {
    pub fn pubkey(stat: Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[SEED_HISTORICAL_AVGS, stat.as_ref()], &crate::ID).0
    }
}
