// pub mod error;
// pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
};
use anchor_lang::InstructionData;
use clockwork_sdk::state::{Thread, ThreadAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod automated_vault {
    use super::*;

    pub fn transfer() -> ProgramResult {
        Ok(())
    }



    pub fn initialize(ctx: Context<Initialize>,thread_id: Vec<u8>) -> Result<()> {
        // Get accounts.
        let system_program = &ctx.accounts.system_program;
        let clockwork_program = &ctx.accounts.clockwork_program;
        let payer = &ctx.accounts.payer;
        let thread = &ctx.accounts.thread;
        let thread_authority = &ctx.accounts.thread_authority;
        let vault = &mut ctx.accounts.vault;

        // 1️⃣ Prepare an instruction to be automated.
        let target_ix = Instruction {
            program_id: ID,
            accounts: crate::accounts::Increment {
                counter: counter.key(),
                thread: thread.key(),
                thread_authority: thread_authority.key(),
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::Increment {}.data(),
        };

        // 2️⃣ Define a trigger for the thread (every 10 secs).
        let trigger = clockwork_sdk::state::Trigger::Cron {
            schedule: "*/10 * * * * * *".into(),
            skippable: true,
        };

        // 3️⃣ Create thread via CPI.
        let bump = *ctx.bumps.get("thread_authority").unwrap();
        clockwork_sdk::cpi::thread_create(
            CpiContext::new_with_signer(
                clockwork_program.to_account_info(),
                clockwork_sdk::cpi::ThreadCreate {
                    payer: payer.to_account_info(),
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
}

/// Seed for deriving the `Vault` account PDA.
pub const SEED_VAULT: &[u8] = b"vault";

/// Seed for thread_authority PDA.
pub const THREAD_AUTHORITY_SEED: &[u8] = b"authority";

#[derive(Accounts)]
#[instruction(thread_id: Vec<u8>)]
pub struct Initialize<'info> {
    /// The vault account to initialize.
    #[account(
        init,
        payer = payer,
        seeds = [SEED_VAULT],
        bump,
        space = 8 + std::mem::size_of::< Vault > (),
    )]
    pub vault: Account<'info, Vault>,

    /// The Clockwork thread program.
    #[account(address = clockwork_sdk::ID)]
    pub clockwork_program: Program<'info, clockwork_sdk::ThreadProgram>,

    /// The signer who will pay to initialize the program.
    /// (not to be confused with the thread executions).
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// Address to assign to the newly created thread.
    #[account(mut, address = Thread::pubkey(thread_authority.key(), thread_id))]
    pub thread: SystemAccount<'info>,

    /// The pda that will own and manage the thread.
    #[account(seeds = [THREAD_AUTHORITY_SEED], bump)]
    pub thread_authority: SystemAccount<'info>,
}
