/**
* Author: Scarcity-pretend (Spxc)
* Created: 04.06.2024
* Description: Simple Solana smart contract which creates a vested fund which will pay out after vesting periode is over.
**/
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    program_pack::Sealed,
    pubkey::Pubkey,
    //sysvar::Sysvar,
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

// Define init function
pub fn init_vesting(
    vesting_state_info: &AccountInfo,
    _vault_info: &AccountInfo,
    funder_info: &AccountInfo,
    recipient_info: &AccountInfo,
    _token_program_info: &AccountInfo,
    amount: u64,
    vesting_end: i64,
    clock: &Clock,
) -> ProgramResult {
    let vesting_state = VestingState {
        is_initialized: false,
        receiver: recipient_info.key.clone(),
        funder: funder_info.key.clone(),
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
    // Implement token transfer logic here

    Ok(())
}

// Define claim function
pub fn claim_vesting(
    vesting_state_info: &AccountInfo,
    _vault_info: &AccountInfo,
    _recipient_info: &AccountInfo,
    _token_program_info: &AccountInfo,
    clock: &Clock,
) -> ProgramResult {
    let vesting_state = VestingState::unpack_from_slice(&vesting_state_info.try_borrow_data()?)?;

    // Verify if timestamp is outside vesting period
    if clock.unix_timestamp <= vesting_state.vesting_end {
        return Err(ProgramError::Custom(0)); // Vesting period has not ended
    }

    // Transfer vested tokens to the recipient
    // Implement token transfer logic here

    // Mark the vesting state as not initialized to prevent further claims
    let mut new_vesting_state = vesting_state;
    new_vesting_state.is_initialized = false;
    new_vesting_state.pack_into_slice(&mut vesting_state_info.try_borrow_mut_data()?);

    Ok(())
}

// Define entrypoint
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Your logic to handle different instructions and dispatch to appropriate functions goes here
    Ok(())
}

