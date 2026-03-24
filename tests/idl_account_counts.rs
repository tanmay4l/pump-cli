mod common;

use common::*;
use pump_cli::core::instructions;

#[test]
fn buy_16() {
    let ix = instructions::build_buy_ix(
        &test_user(),
        &test_mint(),
        1_000_000,
        100_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(ix.accounts.len(), instructions::PUMP_BUY_ACCOUNT_COUNT);
}

#[test]
fn sell_14() {
    let ix = instructions::build_sell_ix(
        &test_user(),
        &test_mint(),
        1_000_000,
        50_000_000,
        &test_creator(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(ix.accounts.len(), instructions::PUMP_SELL_ACCOUNT_COUNT);
}

#[test]
fn create_14() {
    let ix = instructions::build_create_ix(&test_user(), &test_mint(), "T", "T", "https://x.com");
    assert_eq!(ix.accounts.len(), instructions::PUMP_CREATE_ACCOUNT_COUNT);
}

#[test]
fn create_v2_16() {
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
    assert_eq!(
        ix.accounts.len(),
        instructions::PUMP_CREATE_V2_ACCOUNT_COUNT
    );
}

#[test]
fn swap_buy_23() {
    let ix = instructions::build_swap_buy_ix(
        &test_user(),
        &test_user(),
        &test_swap_pool(),
        1_000_000,
        100_000_000,
        &spl_token::id(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(ix.accounts.len(), instructions::PUMP_SWAP_BUY_ACCOUNT_COUNT);
}

#[test]
fn swap_sell_21() {
    let ix = instructions::build_swap_sell_ix(
        &test_user(),
        &test_user(),
        &test_swap_pool(),
        1_000_000,
        50_000_000,
        &spl_token::id(),
        &spl_token::id(),
        &test_fee_recipient(),
    );
    assert_eq!(
        ix.accounts.len(),
        instructions::PUMP_SWAP_SELL_ACCOUNT_COUNT
    );
}
