use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Withdraw amount cannot be less than target")]
    InvalidAmount,
    
    #[msg("Target is not reached yet")]
    TargetNotReached
}