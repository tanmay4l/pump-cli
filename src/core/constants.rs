use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::sync::LazyLock;

pub const PUMP_PROGRAM_ID: Pubkey = pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const PUMP_SWAP_PROGRAM_ID: Pubkey = pubkey!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const PUMP_FEES_PROGRAM_ID: Pubkey = pubkey!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const TOKEN_2022_PROGRAM_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// Mayhem program: PDA derivation domain for global_params, sol_vault, mayhem_state.
pub const MAYHEM_PROGRAM_ID: Pubkey = pubkey!("MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e");

pub const PUMP_GLOBAL: Pubkey = pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf");
pub const FEE_RECIPIENT: Pubkey = pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");
pub const EVENT_AUTHORITY: Pubkey = pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1");
pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const PROTOCOL_FEE_RECIPIENT: Pubkey = pubkey!("62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV");

pub const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
pub const RENT_SYSVAR_ID: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

pub static PUMP_SWAP_EVENT_AUTHORITY: LazyLock<Pubkey> = LazyLock::new(|| {
    let (pda, _) = Pubkey::find_program_address(&[b"__event_authority"], &PUMP_SWAP_PROGRAM_ID);
    pda
});

/// Pump program fee_config PDA: seeds = ["fee_config", PUMP_PROGRAM_ID]
pub static PUMP_FEE_CONFIG: LazyLock<Pubkey> = LazyLock::new(|| {
    let (pda, _) = Pubkey::find_program_address(
        &[b"fee_config", PUMP_PROGRAM_ID.as_ref()],
        &PUMP_FEES_PROGRAM_ID,
    );
    pda
});

/// Pump program global_volume_accumulator PDA: seeds = ["global_volume_accumulator"]
pub static PUMP_GLOBAL_VOLUME_ACCUMULATOR: LazyLock<Pubkey> = LazyLock::new(|| {
    let (pda, _) = Pubkey::find_program_address(&[b"global_volume_accumulator"], &PUMP_PROGRAM_ID);
    pda
});

/// PumpSwap fee_config PDA: seeds = ["fee_config", PUMP_SWAP_PROGRAM_ID]
pub static PUMP_SWAP_FEE_CONFIG: LazyLock<Pubkey> = LazyLock::new(|| {
    let (pda, _) = Pubkey::find_program_address(
        &[b"fee_config", PUMP_SWAP_PROGRAM_ID.as_ref()],
        &PUMP_FEES_PROGRAM_ID,
    );
    pda
});

/// PumpSwap global_volume_accumulator PDA: seeds = ["global_volume_accumulator"]
pub static PUMP_SWAP_GLOBAL_VOLUME_ACCUMULATOR: LazyLock<Pubkey> = LazyLock::new(|| {
    let (pda, _) =
        Pubkey::find_program_address(&[b"global_volume_accumulator"], &PUMP_SWAP_PROGRAM_ID);
    pda
});

pub const TOKEN_DECIMALS: u8 = 6;
pub const BONDING_CURVE_FEE_BPS: u64 = 100;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const PUMP_SWAP_TOTAL_FEE_BPS: u64 = 25;
