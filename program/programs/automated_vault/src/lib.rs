pub mod id;
mod state;
use state::vault::Vault;
mod error;
use error::ErrorCode;
pub use id::ID;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{ instruction::Instruction,native_token::LAMPORTS_PER_SOL,system_program };
use anchor_lang::InstructionData;
use clockwork_sdk::state::{Thread, ThreadAccount};


const RECURRING_TRANSFER: i64 = 1;
/// Seed for deriving the `Vault` account PDA.
pub const SEED_VAULT: &[u8] = b"vault";
/// Seed for thread_authority PDA.
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[program]
pub mod automated_vault {
    use super::*;
    
    pub fn initialize_vault(
        ctx: Context<Initialize>,
        thread_id: Vec<u8>,
        balance:i64,
        target:i64,
        label:String,
    ) -> Result<()> {
        // Get accounts.
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let owner = &ctx.accounts.owner;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let vault = &mut ctx.accounts.vault;

        //Assign values to the vault
        vault.thread_id = thread_id.clone();
        vault.owner = *owner.key;
        vault.balance = balance ;
        vault.target = target;
        vault.label = label;
        vault.updated_at = Clock::get()?.unix_timestamp;

        // 1️⃣ Prepare an instruction to be automated.
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::RecurringTransfer {
                vault: vault.key(),
                thread: thread.key(),
                thread_authority: thread_authority.key(),
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::RecurringTransfer {
                _thread_id: thread_id.clone(),
            }.data(),
        };

        // 2️⃣ Define a trigger for the thread (every 10 secs).
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/10 * * * * * *".into(), // every 10 secs
            skippable: true,
        };

        // 3️⃣ Create thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: owner.to_account_info(),
                    system_program: system_program.to_account_info(),
                    thread: thread.to_account_info(),
                    authority: thread_authority.to_account_info(),
                },
                &[&[THREAD_AUTHORITY_SEED, &[bump]]],
            ),
            LAMPORTS_PER_SOL,       // amount
            thread_id,              // id
            vec![target_ix.into()], // instructions
            trigger,                // trigger
        )?;

        Ok(())
    }

    pub fn recurring_transfer(ctx: Context<RecurringTransfer>, _thread_id: Vec<u8>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    let current_time = Clock::get()?.unix_timestamp;

    // Calculate minutes passed since last update
    let minutes_passed = (current_time - vault.updated_at) / 60;

    // Check if a minute or more has passed since the last update and vault balance is less than the target
    if minutes_passed >= 1 && vault.balance < vault.target {
        // Add recurring transfer to the balance

        //We are making sure to multiply the minutes passed even if the function is called very less often more than a minute
        vault.balance += RECURRING_TRANSFER * minutes_passed as i64;

        // Limit the balance to the target
        if vault.balance > vault.target {
            vault.balance = vault.target;
            msg!("The Vault balance is now equal to the target");
        }

        // Update the last updated timestamp and report the balance
        vault.updated_at = current_time;
        msg!("The vault balance is {} and updated at {}", vault.balance, current_time);
    }

    Ok(())
    }

    pub fn withdraw(ctx: Context<WithdrawBalance>,_thread_id: Vec<u8>,amount:i64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        if amount == vault.balance && amount == vault.target {
            vault.balance = 0;
        }

        else if amount < vault.target {
            return Err(error!(ErrorCode::TargetNotReached));
        }

        else if amount > vault.balance || amount < 0 {  {
            return Err(error!(ErrorCode::InvalidAmount));
        }} 

        Ok(())
    }

    pub fn close_vault(ctx: Context<CloseVault>, _thread_id: Vec<u8>) -> Result<()> {

        let clockwork_program = &ctx.accounts.clockwork_program;
        let owner = &ctx.accounts.owner;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;

        // Delete thread via CPI and withdraw the remaining balance
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_delete(CpiContext::new_with_signer(
            clockwork_program.to_account_info(),
            clockwork_sdk::cpi::ThreadDelete {
                authority: thread_authority.to_account_info(),
                close_to: owner.to_account_info(),
                thread: thread.to_account_info(),
            },
            &[&[THREAD_AUTHORITY_SEED, &[bump]]],
        ))?;
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct Initialize<'info> {
    /// The vault account to initialize.
    #[account(
        init,
        payer = owner,
        seeds = [SEED_VAULT,thread_id.as_ref()],
        bump,
        space = 8 + std::mem::size_of::<Vault> (),
    )]
    pub vault: Account<'info, Vault>,

    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Address to assign to the newly created thread.
    #[account(mut, address = Thread::pubkey(thread_authority.key(), thread_id))]
    pub thread: SystemAccount<'info>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct WithdrawBalance<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [SEED_VAULT, thread_id.as_ref()], bump)]
    pub vault: Account<'info, Vault>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct RecurringTransfer<'info> {
    #[account(mut, seeds = [SEED_VAULT, thread_id.as_ref()], bump)]
    pub vault: Account<'info, Vault>,

    #[account(signer, constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,
    
    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,
}

#[derive(Accounts)]
#[instruction(thread_id : Vec<u8>)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_VAULT, thread_id.as_ref()],
        bump,
        close = owner
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut, address = thread.pubkey(), constraint = thread.authority.eq(&thread_authority.key()))]
    pub thread: Account<'info, Thread>,

    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,

    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,
}

