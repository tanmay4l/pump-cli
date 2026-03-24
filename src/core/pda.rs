use solana_sdk::pubkey::Pubkey;

use super::constants::{MAYHEM_PROGRAM_ID, PUMP_PROGRAM_ID, PUMP_SWAP_PROGRAM_ID};

pub fn bonding_curve_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"bonding-curve", mint.as_ref()], &PUMP_PROGRAM_ID)
}

pub fn mint_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"mint-authority"], &PUMP_PROGRAM_ID)
}

pub fn creator_vault_pda(creator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"creator-vault", creator.as_ref()], &PUMP_PROGRAM_ID)
}

pub fn pool_authority_pda(base_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool-authority", base_mint.as_ref()], &PUMP_PROGRAM_ID)
}

pub fn pump_swap_pool_pda(
    index: u16,
    creator: &Pubkey,
    base_mint: &Pubkey,
    quote_mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"pool",
            &index.to_le_bytes(),
            creator.as_ref(),
            base_mint.as_ref(),
            quote_mint.as_ref(),
        ],
        &PUMP_SWAP_PROGRAM_ID,
    )
}

pub fn pump_swap_global_config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"global_config"], &PUMP_SWAP_PROGRAM_ID)
}

pub fn pump_swap_creator_vault_pda(coin_creator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"creator_vault", coin_creator.as_ref()],
        &PUMP_SWAP_PROGRAM_ID,
    )
}

pub fn user_volume_accumulator_pda(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PUMP_PROGRAM_ID,
    )
}

pub fn pump_swap_user_volume_accumulator_pda(user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PUMP_SWAP_PROGRAM_ID,
    )
}

// These three PDAs derive under MAYHEM_PROGRAM_ID, not PUMP_PROGRAM_ID.
pub fn global_params_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"global-params"], &MAYHEM_PROGRAM_ID)
}
pub fn sol_vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"sol-vault"], &MAYHEM_PROGRAM_ID)
}
pub fn mayhem_state_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"mayhem-state", mint.as_ref()], &MAYHEM_PROGRAM_ID)
}
