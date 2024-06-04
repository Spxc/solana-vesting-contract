/**
* Author: Scarcity-pretend (Spxc)
* Created: 04.06.2024
* Description: Simple Solana smart contract which creates a vested fund which will pay out after vesting periode is over.
**/

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer as SplTransfer};
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::FromPrimitive;

declare_id!("DFGuapfSuXhUpU9V1yNbMUZ76tReRqeY8F4byVTcUWV8");

// define program states
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VestingState {
    is_initialized: bool,
    receiver: Pubkey,
    funder: Pubkey,
    amount: u64,
    vesting_start: i64,
    vesting_end: i64,
}

// define init types
#[derive(Accounts)]
pub struct InitializeVesting<'info> {
    #[account(init, payer = funder, space = 8 + 128)]
    pub vesting_state: Account<'info, VestingState>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

// define claiming types
#[derive(Accounts)]
pub struct ClaimVesting<'info> {
    #[account(mut)]
    pub vesting_state: Account<'info, VestingState>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub recipient: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(address = sysvar::clock::ID)]
    pub clock: Sysvar<'info, Clock>,
}

// program start
#[program]
mod vesting {
    use super::*;

    // fnc: init program and set vested funds
    pub fn init_vesting(
        ctx: Context<InitializeVesting>,
        amount: u64,
        vesting_end: i64,
    ) -> ProgramResult {
        let vesting_state = &mut ctx.accounts.vesting_state;

        // validate if the program has been initialized before
        if vesting_state.is_initialized {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // set program boundaries
        vesting_state.is_initialized = true;
        vesting_state.receiver = ctx.accounts.recipient.key();
        vesting_state.funder = ctx.accounts.funder.key();
        vesting_state.amount = amount;
        vesting_state.vesting_start = ctx.accounts.clock.unix_timestamp;
        vesting_state.vesting_end = vesting_end;

        // transfer vested tokens to the vault account
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.funder.to_account_info().clone(),
            to: ctx.accounts.vault.to_account_info().clone(),
            authority: ctx.accounts.funder.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // fnc: claim vested funds
    pub fn claim_vesting(ctx: Context<ClaimVesting>) -> ProgramResult {
        let vesting_state = &mut ctx.accounts.vesting_state;
        let clock = &ctx.accounts.clock;

        // verify is timestamp is outside vesting periode
        if clock.unix_timestamp <= vesting_state.vesting_end {
            return Err(ProgramError::Custom(0)); // Vesting period has not ended
        }

        // transfer vested tokens to the recipient
        let cpi_accounts = SplTransfer {
            from: ctx.accounts.vault.to_account_info().clone(),
            to: ctx.accounts.recipient.to_account_info().clone(),
            authority: ctx.accounts.vault.to_account_info().clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let seeds = &[vesting_state.to_account_info().key.as_ref()];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, vesting_state.amount)?;

        // mark the vesting state as not initialized to prevent further claims
        vesting_state.is_initialized = false;

        Ok(())
    }
}
