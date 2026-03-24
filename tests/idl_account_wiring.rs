mod common;

use common::*;
use pump_cli::core::constants;
use pump_cli::core::instructions;
use pump_cli::core::pump_swap::SwapPool;
use solana_sdk::pubkey::Pubkey;

#[test]
fn buy_account_flags() {
    let user = test_user();
    let ix = instructions::build_buy_ix(
        &user,
        &test_mint(),
        1_000_000,
        100_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert!(!ix.accounts[0].is_writable && !ix.accounts[0].is_signer);
    assert!(ix.accounts[1].is_writable);
    assert!(!ix.accounts[2].is_writable);
    assert!(ix.accounts[3].is_writable);
    assert!(ix.accounts[4].is_writable);
    assert!(ix.accounts[5].is_writable);
    assert!(ix.accounts[6].is_writable && ix.accounts[6].is_signer);
    assert_eq!(ix.accounts[6].pubkey, user);
    assert!(!ix.accounts[7].is_writable);
    assert!(!ix.accounts[8].is_writable);
    assert!(ix.accounts[9].is_writable);
    assert!(!ix.accounts[10].is_writable);
    assert!(!ix.accounts[11].is_writable);
    assert!(!ix.accounts[12].is_writable);
    assert!(ix.accounts[13].is_writable);
    assert!(!ix.accounts[14].is_writable);
    assert!(!ix.accounts[15].is_writable);
}

#[test]
fn sell_account_flags() {
    let ix = instructions::build_sell_ix(
        &test_user(),
        &test_mint(),
        1_000_000,
        50_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert!(!ix.accounts[0].is_writable);
    assert!(ix.accounts[1].is_writable);
    assert!(!ix.accounts[2].is_writable);
    assert!(ix.accounts[6].is_writable && ix.accounts[6].is_signer);
    assert!(!ix.accounts[7].is_writable);
    assert!(ix.accounts[8].is_writable);
    assert!(!ix.accounts[9].is_writable);
    assert!(!ix.accounts[12].is_writable);
    assert!(!ix.accounts[13].is_writable);
}

#[test]
fn buy_data_layout() {
    let ix = instructions::build_buy_ix(
        &test_user(),
        &test_mint(),
        42_000_000,
        1_500_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(ix.data.len(), 25);
    assert_eq!(&ix.data[0..8], &instructions::PUMP_BUY_DISCRIMINATOR);
    assert_eq!(
        u64::from_le_bytes(ix.data[8..16].try_into().unwrap()),
        42_000_000
    );
    assert_eq!(
        u64::from_le_bytes(ix.data[16..24].try_into().unwrap()),
        1_500_000_000
    );
    assert_eq!(ix.data[24], 0);
}

#[test]
fn sell_data_layout() {
    let ix = instructions::build_sell_ix(
        &test_user(),
        &test_mint(),
        10_000_000,
        500_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(ix.data.len(), 24);
    assert_eq!(&ix.data[0..8], &instructions::PUMP_SELL_DISCRIMINATOR);
    assert_eq!(
        u64::from_le_bytes(ix.data[8..16].try_into().unwrap()),
        10_000_000
    );
    assert_eq!(
        u64::from_le_bytes(ix.data[16..24].try_into().unwrap()),
        500_000_000
    );
}

#[test]
fn create_data_has_creator() {
    let user = test_user();
    let ix = instructions::build_create_ix(&user, &test_mint(), "Test", "T", "https://x.com");
    assert_eq!(&ix.data[ix.data.len() - 32..], user.as_ref());
}

#[test]
fn create_v2_data_has_flags() {
    let ix = instructions::build_create_v2_ix(
        &test_user(),
        &test_mint(),
        "T",
        "T",
        "https://x.com",
        &spl_token::id(),
        true,
        true,
    );
    let len = ix.data.len();
    assert_eq!(ix.data[len - 2], 1);
    assert_eq!(ix.data[len - 1], 1);
}

#[test]
fn create_v2_fixed_addresses() {
    let ix = instructions::build_create_v2_ix(
        &test_user(),
        &test_mint(),
        "T",
        "T",
        "https://x.com",
        &constants::TOKEN_2022_PROGRAM_ID,
        false,
        false,
    );
    assert_eq!(
        ix.accounts[6].pubkey,
        solana_sdk::pubkey!("11111111111111111111111111111111")
    );
    assert_eq!(ix.accounts[7].pubkey, constants::TOKEN_2022_PROGRAM_ID);
    assert_eq!(ix.accounts[8].pubkey, spl_associated_token_account::id());
    assert_eq!(ix.accounts[9].pubkey, constants::MAYHEM_PROGRAM_ID);
    assert!(ix.accounts[9].is_writable);
    assert_eq!(ix.accounts[14].pubkey, constants::EVENT_AUTHORITY);
    assert_eq!(ix.accounts[15].pubkey, constants::PUMP_PROGRAM_ID);
}

#[test]
fn create_v2_forces_token2022() {
    let ix = instructions::build_create_v2_ix(
        &test_user(),
        &test_mint(),
        "T",
        "T",
        "https://x.com",
        &spl_token::id(),
        false,
        false,
    );
    assert_eq!(ix.accounts[7].pubkey, constants::TOKEN_2022_PROGRAM_ID);
}

#[test]
fn create_v2_pda_domains() {
    let ix = instructions::build_create_v2_ix(
        &test_user(),
        &test_mint(),
        "T",
        "T",
        "https://x.com",
        &constants::TOKEN_2022_PROGRAM_ID,
        false,
        false,
    );
    let exp_gp = Pubkey::find_program_address(&[b"global-params"], &constants::MAYHEM_PROGRAM_ID).0;
    let exp_sv = Pubkey::find_program_address(&[b"sol-vault"], &constants::MAYHEM_PROGRAM_ID).0;
    let exp_ms = Pubkey::find_program_address(
        &[b"mayhem-state", test_mint().as_ref()],
        &constants::MAYHEM_PROGRAM_ID,
    )
    .0;
    assert_eq!(ix.accounts[10].pubkey, exp_gp);
    assert_eq!(ix.accounts[11].pubkey, exp_sv);
    assert_eq!(ix.accounts[12].pubkey, exp_ms);
}

#[test]
fn create_v2_all_flags() {
    let ix = instructions::build_create_v2_ix(
        &test_user(),
        &test_mint(),
        "T",
        "T",
        "https://x.com",
        &constants::TOKEN_2022_PROGRAM_ID,
        true,
        true,
    );
    assert!(ix.accounts[0].is_writable && ix.accounts[0].is_signer);
    assert!(!ix.accounts[1].is_writable && !ix.accounts[1].is_signer);
    assert!(ix.accounts[2].is_writable);
    assert!(ix.accounts[3].is_writable);
    assert!(!ix.accounts[4].is_writable);
    assert!(ix.accounts[5].is_writable && ix.accounts[5].is_signer);
    assert!(!ix.accounts[6].is_writable);
    assert!(!ix.accounts[7].is_writable);
    assert!(!ix.accounts[8].is_writable);
    assert!(ix.accounts[9].is_writable);
    assert!(!ix.accounts[10].is_writable);
    assert!(ix.accounts[11].is_writable);
    assert!(ix.accounts[12].is_writable);
    assert!(ix.accounts[13].is_writable);
    assert!(!ix.accounts[14].is_writable);
    assert!(!ix.accounts[15].is_writable);
}

#[test]
fn swap_pool_mayhem_cashback() {
    let mut data = vec![0u8; 237];
    data[235] = 1;
    data[236] = 1;
    let pool = SwapPool::deserialize(&data).unwrap();
    assert!(pool.is_mayhem_mode);
    assert!(pool.is_cashback_coin);
}

#[test]
fn swap_buy_dynamic_recipient() {
    let r = Pubkey::new_unique();
    let ix = instructions::build_swap_buy_ix(
        &test_user(),
        &test_user(),
        &test_swap_pool(),
        1_000_000,
        100_000_000,
        &spl_token::id(),
        &spl_token::id(),
        &r,
    );
    assert_eq!(ix.accounts[9].pubkey, r);
    assert!(!ix.accounts[9].is_writable);
    let ata =
        pump_cli::core::token_accounts::get_ata(&r, &test_swap_pool().quote_mint, &spl_token::id());
    assert_eq!(ix.accounts[10].pubkey, ata);
}

#[test]
fn swap_sell_dynamic_recipient() {
    let r = Pubkey::new_unique();
    let ix = instructions::build_swap_sell_ix(
        &test_user(),
        &test_user(),
        &test_swap_pool(),
        1_000_000,
        50_000_000,
        &spl_token::id(),
        &spl_token::id(),
        &r,
    );
    assert_eq!(ix.accounts[9].pubkey, r);
    let ata =
        pump_cli::core::token_accounts::get_ata(&r, &test_swap_pool().quote_mint, &spl_token::id());
    assert_eq!(ix.accounts[10].pubkey, ata);
}

#[test]
fn token2022_extended_data() {
    let mut data = vec![0u8; 165];
    let mint_bytes = [42u8; 32];
    data[0..32].copy_from_slice(&mint_bytes);
    data[64..72].copy_from_slice(&1_000_000u64.to_le_bytes());

    let mut extended = data.clone();
    extended.extend_from_slice(&[0u8; 100]);

    for d in &[&data[..], &extended[..]] {
        assert!(d.len() >= 72);
        assert_eq!(Pubkey::try_from(&d[0..32]).unwrap().to_bytes(), mint_bytes);
        assert_eq!(u64::from_le_bytes(d[64..72].try_into().unwrap()), 1_000_000);
    }
}
