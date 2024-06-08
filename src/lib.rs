/**
 * @title Vesting Smart Contract
 * @version 1.0.0
 * @date 2024-06-08
 * @license MIT
 *
 * @summary
 * This smart contract implements a simple vesting mechanism on the Solana blockchain.
 * It allows a funder to lock a specific amount of tokens in a vault, which will be released
 * to a designated recipient after a predefined vesting period.
 *
 * @details
 * - The `init_vesting` function initializes the vesting schedule, transferring tokens from the funder to a vault.
 * - The `claim_vesting` function allows the recipient to claim the vested tokens once the vesting period has ended.
 * - The vesting schedule is immutable once set; neither the amount nor the recipient can be changed.
 * - Token transfers are handled using the SPL Token program.
 *
 * @authors
 * - Scarcity-pretend (Spxc)
 *
 * @changelog
 * - 2024-06-08: Added vesting and claim functionality.
 * - 2024-06-04: Initial version
 */
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

// Define program states
#[derive(Debug)]
pub struct VestingState {
    pub is_initialized: bool,
    pub receiver: Pubkey,
    pub funder: Pubkey,
    pub amount: u64,
    pub vesting_start: i64,
    pub vesting_end: i64,
}

impl Sealed for VestingState {}
impl Pack for VestingState {
    const LEN: usize = 97;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let receiver_bytes: [u8; 32] = src[0..32].try_into().unwrap();
        let funder_bytes: [u8; 32] = src[32..64].try_into().unwrap();

        Ok(VestingState {
            is_initialized: true,
            receiver: Pubkey::from(receiver_bytes),
            funder: Pubkey::from(funder_bytes),
            amount: u64::from_le_bytes((&src[64..72]).try_into().unwrap()),
            vesting_start: i64::from_le_bytes((&src[72..80]).try_into().unwrap()),
            vesting_end: i64::from_le_bytes((&src[80..88]).try_into().unwrap()),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[0..32].copy_from_slice(self.receiver.as_ref());
        dst[32..64].copy_from_slice(self.funder.as_ref());
        dst[64..72].copy_from_slice(&self.amount.to_le_bytes());
        dst[72..80].copy_from_slice(&self.vesting_start.to_le_bytes());
        dst[80..88].copy_from_slice(&self.vesting_end.to_le_bytes());
    }
}

entrypoint!(process_instruction);

impl IsInitialized for VestingState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

/**
 * Initializes a vesting schedule.
 *
 * This function transfers the specified amount of tokens from the funder's account
 * to a vault account and records the vesting details in the vesting state account.
 * The vesting state includes the recipient, funder, amount, vesting start and end times.
 * The vesting start time is set to the current timestamp.
 *
 * Accounts expected by this instruction:
 * 0. `[writable]` The vesting state account to be initialized.
 * 1. `[writable]` The vault account to hold the vested tokens.
 * 2. `[signer]` The funder's account, from which tokens will be transferred.
 * 3. `[]` The recipient's account, which will receive the tokens after vesting.
 * 4. `[]` The SPL token program account.
 * 5. `[]` The Rent sysvar.
 * 6. `[]` The Clock sysvar.
 *
 * Parameters:
 * - `amount`: The amount of tokens to be vested.
 * - `vesting_end`: The Unix timestamp when the vesting period ends.
 */
pub fn init_vesting(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    vesting_end: i64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vesting_state_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let funder_info = next_account_info(account_info_iter)?;
    let recipient_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
    let clock = Clock::from_account_info(next_account_info(account_info_iter)?)?;

    if !rent.is_exempt(vesting_state_info.lamports(), vesting_state_info.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }

    let mut vesting_state = VestingState {
        is_initialized: false,
        receiver: *recipient_info.key,
        funder: *funder_info.key,
        amount,
        vesting_start: clock.unix_timestamp,
        vesting_end,
    };

    // Validate if the program has been initialized before
    if vesting_state_info
        .try_borrow_data()?
        .iter()
        .all(|&byte| byte == 0)
    {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Write vesting state to account
    vesting_state.pack_into_slice(&mut vesting_state_info.try_borrow_mut_data()?);

    // Transfer vested tokens to the vault account
    let transfer_ix = spl_token::instruction::transfer(
        token_program_info.key,
        funder_info.key,
        vault_info.key,
        funder_info.key,
        &[],
        amount,
    )?;
    invoke(
        &transfer_ix,
        &[
            funder_info.clone(),
            vault_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    Ok(())
}

/**
 * Claims the vested tokens.
 *
 * This function allows the recipient to claim the vested tokens after the vesting period has ended.
 * It checks the current timestamp to ensure the vesting period is over, then transfers the tokens
 * from the vault account to the recipient's account. The vesting state is marked as uninitialized
 * to prevent further claims.
 *
 * Accounts expected by this instruction:
 * 0. `[writable]` The vesting state account.
 * 1. `[writable]` The vault account holding the vested tokens.
 * 2. `[writable]` The recipient's account, which will receive the tokens.
 * 3. `[]` The SPL token program account.
 * 4. `[]` The Clock sysvar.
 */
pub fn claim_vesting(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let vesting_state_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let recipient_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let clock = Clock::from_account_info(next_account_info(account_info_iter)?)?;

    let vesting_state = VestingState::unpack_from_slice(&vesting_state_info.try_borrow_data()?)?;

    // Verify if timestamp is outside vesting period
    if clock.unix_timestamp <= vesting_state.vesting_end {
        return Err(ProgramError::Custom(0)); // Vesting period has not ended
    }

    // Transfer vested tokens to the recipient
    let transfer_ix = spl_token::instruction::transfer(
        token_program_info.key,
        vault_info.key,
        recipient_info.key,
        vesting_state_info.key,
        &[],
        vesting_state.amount,
    )?;
    invoke_signed(
        &transfer_ix,
        &[
            vault_info.clone(),
            recipient_info.clone(),
            token_program_info.clone(),
        ],
        &[&[b"vesting", &[vesting_state_info.data_len() as u8]]], // Update seeds as needed
    )?;

    // Mark the vesting state as not initialized to prevent further claims
    let mut new_vesting_state = vesting_state;
    new_vesting_state.is_initialized = false;
    new_vesting_state.pack_into_slice(&mut vesting_state_info.try_borrow_mut_data()?);

    Ok(())
}

/**
 * Processes instructions for the smart contract.
 *
 * This function is the main entry point for the program. It dispatches calls to the appropriate
 * function based on the instruction data. The first byte of the instruction data specifies the
 * instruction to be executed.
 *
 * Parameters:
 * - `_program_id`: The program ID.
 * - `_accounts`: The accounts required for the instruction.
 * - `_instruction_data`: The instruction data.
 *
 * Supported instructions:
 * - `0`: Initialize vesting (calls `init_vesting`).
 * - `1`: Claim vesting (calls `claim_vesting`).
 */
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = instruction_data[0];

    match instruction {
        0 => {
            let (amount, vesting_end) = unpack_init_instruction(instruction_data)?;
            init_vesting(program_id, accounts, amount, vesting_end)
        }
        1 => claim_vesting(program_id, accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/**
 * Unpacks initialization instruction data.
 *
 * This helper function unpacks the amount and vesting end timestamp from the provided
 * instruction data. It expects the data to be exactly 16 bytes long: 8 bytes for the amount
 * and 8 bytes for the vesting end timestamp.
 *
 * Parameters:
 * - `data`: The instruction data.
 *
 * Returns:
 * - A tuple containing the amount and the vesting end timestamp.
 */
fn unpack_init_instruction(data: &[u8]) -> Result<(u64, i64), ProgramError> {
    if data.len() != 16 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let vesting_end = i64::from_le_bytes(data[8..16].try_into().unwrap());
    Ok((amount, vesting_end))
}
