use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Amount must be greater than required deposit")]
    InsufficientBalance,

    #[msg("Withdraw amount cannot be less than deposit")]
    AmountTooBig,
}