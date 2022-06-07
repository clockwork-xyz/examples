use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Transfer rate is greater than deposit amount")]
    InvalidTransferRate,
}
