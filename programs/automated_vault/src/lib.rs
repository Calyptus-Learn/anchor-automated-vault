pub mod error;
pub mod state;

use anchor_lang::prelude::*;
// use anchor_lang::solana_program::{
//     instruction::Instruction, native_token::LAMPORTS_PER_SOL, system_program,
// };
// use anchor_lang::InstructionData;
// use clockwork_sdk::state::{Thread, ThreadAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod automated_vault {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

