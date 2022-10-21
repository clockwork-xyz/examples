use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Insuffiscient amount locked to withdraw")]
    InsuffiscientAmountLocked,
}
