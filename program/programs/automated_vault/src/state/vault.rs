use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub owner: Pubkey,    // Public key of the user's account
    pub balance: u64,     // Current balance of the vault
    pub target: u64,      // Target amount for the vault
    pub label: String,    // Label or purpose of the vault
}