use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub owner: Pubkey,    // Public key of the user's account
    pub balance: i64,     // Current balance of the vault
    pub target: i64,      // Target amount for the vault
    pub label: String,    // Label or purpose of the vault
    pub thread_id :Vec<u8>, // thread_id for the vault 
    pub updated_at: i64,  // Last updated timestamp
}