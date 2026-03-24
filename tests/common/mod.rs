use pump_cli::core::constants;
use pump_cli::core::pump_swap::SwapPool;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn test_fee_recipient() -> Pubkey {
    constants::FEE_RECIPIENT
}

pub fn test_user() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111112").unwrap()
}

pub fn test_mint() -> Pubkey {
    Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()
}

pub fn test_creator() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111113").unwrap()
}

pub fn test_swap_pool() -> SwapPool {
    SwapPool {
        pool_bump: 255,
        index: 0,
        creator: test_creator(),
        base_mint: test_mint(),
        quote_mint: Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap(),
        lp_mint: Pubkey::default(),
        pool_base_token_account: Pubkey::default(),
        pool_quote_token_account: Pubkey::default(),
        lp_supply: 0,
        coin_creator: test_creator(),
        is_mayhem_mode: false,
        is_cashback_coin: false,
    }
}
