use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use super::constants::{
    EVENT_AUTHORITY, MAYHEM_PROGRAM_ID, PUMP_FEES_PROGRAM_ID, PUMP_FEE_CONFIG, PUMP_GLOBAL,
    PUMP_GLOBAL_VOLUME_ACCUMULATOR, PUMP_PROGRAM_ID, PUMP_SWAP_EVENT_AUTHORITY,
    PUMP_SWAP_FEE_CONFIG, PUMP_SWAP_GLOBAL_VOLUME_ACCUMULATOR, PUMP_SWAP_PROGRAM_ID,
    RENT_SYSVAR_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
};
use super::pda;
use super::token_accounts;

/// Anchor discriminator: first 8 bytes of sha256("global:<method_name>").
pub fn discriminator(method: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{method}"));
    let hash = hasher.finalize();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}

fn borsh_string(s: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + s.len());
    buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
    buf
}

#[allow(dead_code)]
pub const PUMP_BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
#[allow(dead_code)]
pub const PUMP_SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
#[allow(dead_code)]
pub const PUMP_CREATE_DISCRIMINATOR: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
#[allow(dead_code)]
pub const PUMP_CREATE_V2_DISCRIMINATOR: [u8; 8] = [214, 144, 76, 236, 95, 139, 49, 180];
#[allow(dead_code)]
pub const PUMP_SWAP_BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
#[allow(dead_code)]
pub const PUMP_SWAP_SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];

#[allow(dead_code)]
pub const PUMP_BUY_ACCOUNT_COUNT: usize = 16;
#[allow(dead_code)]
pub const PUMP_SELL_ACCOUNT_COUNT: usize = 14;
#[allow(dead_code)]
pub const PUMP_CREATE_ACCOUNT_COUNT: usize = 14;
#[allow(dead_code)]
pub const PUMP_CREATE_V2_ACCOUNT_COUNT: usize = 16;
#[allow(dead_code)]
pub const PUMP_SWAP_BUY_ACCOUNT_COUNT: usize = 23;
#[allow(dead_code)]
pub const PUMP_SWAP_SELL_ACCOUNT_COUNT: usize = 21;

pub fn build_buy_ix(
    user: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    max_sol_cost: u64,
    creator: &Pubkey,
    token_program: &Pubkey,
    fee_recipient: &Pubkey,
) -> Instruction {
    let (bonding_curve, _) = pda::bonding_curve_pda(mint);
    let associated_bonding_curve = token_accounts::get_ata(&bonding_curve, mint, token_program);
    let associated_user = token_accounts::get_ata(user, mint, token_program);
    let (creator_vault, _) = pda::creator_vault_pda(creator);
    let (user_volume_acc, _) = pda::user_volume_accumulator_pda(user);

    let mut data = Vec::with_capacity(25);
    data.extend_from_slice(&discriminator("buy"));
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&max_sol_cost.to_le_bytes());
    data.push(0);

    Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(PUMP_GLOBAL, false),
            AccountMeta::new(*fee_recipient, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new(associated_user, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new(creator_vault, false),
            AccountMeta::new_readonly(EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_PROGRAM_ID, false),
            AccountMeta::new_readonly(*PUMP_GLOBAL_VOLUME_ACCUMULATOR, false),
            AccountMeta::new(user_volume_acc, false),
            AccountMeta::new_readonly(*PUMP_FEE_CONFIG, false),
            AccountMeta::new_readonly(PUMP_FEES_PROGRAM_ID, false),
        ],
        data,
    }
}

pub fn build_sell_ix(
    user: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    min_sol_output: u64,
    creator: &Pubkey,
    token_program: &Pubkey,
    fee_recipient: &Pubkey,
) -> Instruction {
    let (bonding_curve, _) = pda::bonding_curve_pda(mint);
    let associated_bonding_curve = token_accounts::get_ata(&bonding_curve, mint, token_program);
    let associated_user = token_accounts::get_ata(user, mint, token_program);
    let (creator_vault, _) = pda::creator_vault_pda(creator);

    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(&discriminator("sell"));
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(&min_sol_output.to_le_bytes());

    Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new_readonly(PUMP_GLOBAL, false),
            AccountMeta::new(*fee_recipient, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new(associated_user, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(creator_vault, false),
            AccountMeta::new_readonly(*token_program, false),
            AccountMeta::new_readonly(EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_PROGRAM_ID, false),
            AccountMeta::new_readonly(*PUMP_FEE_CONFIG, false),
            AccountMeta::new_readonly(PUMP_FEES_PROGRAM_ID, false),
        ],
        data,
    }
}

pub fn build_create_ix(
    user: &Pubkey,
    mint: &Pubkey,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Instruction {
    let (mint_authority, _) = pda::mint_authority_pda();
    let (bonding_curve, _) = pda::bonding_curve_pda(mint);
    let associated_bonding_curve =
        spl_associated_token_account::get_associated_token_address(&bonding_curve, mint);
    let metadata = metaplex_metadata_pda(mint);

    let mut data = Vec::new();
    data.extend_from_slice(&discriminator("create"));
    data.extend_from_slice(&borsh_string(name));
    data.extend_from_slice(&borsh_string(symbol));
    data.extend_from_slice(&borsh_string(uri));
    data.extend_from_slice(user.as_ref());

    Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*mint, true),
            AccountMeta::new_readonly(mint_authority, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new_readonly(PUMP_GLOBAL, false),
            AccountMeta::new_readonly(METAPLEX_TOKEN_METADATA_ID, false),
            AccountMeta::new(metadata, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(RENT_SYSVAR_ID, false),
            AccountMeta::new_readonly(EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_PROGRAM_ID, false),
        ],
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_create_v2_ix(
    user: &Pubkey,
    mint: &Pubkey,
    name: &str,
    symbol: &str,
    uri: &str,
    token_program: &Pubkey,
    is_mayhem_mode: bool,
    is_cashback_enabled: bool,
) -> Instruction {
    let _ = token_program;
    let tp = TOKEN_2022_PROGRAM_ID;

    let (mint_authority, _) = pda::mint_authority_pda();
    let (bonding_curve, _) = pda::bonding_curve_pda(mint);
    let associated_bonding_curve = token_accounts::get_ata(&bonding_curve, mint, &tp);
    let (global_params, _) = pda::global_params_pda();
    let (sol_vault, _) = pda::sol_vault_pda();
    let (mayhem_state, _) = pda::mayhem_state_pda(mint);
    let mayhem_token_vault = token_accounts::get_ata(&mayhem_state, mint, &tp);

    let mut data = Vec::new();
    data.extend_from_slice(&discriminator("create_v2"));
    data.extend_from_slice(&borsh_string(name));
    data.extend_from_slice(&borsh_string(symbol));
    data.extend_from_slice(&borsh_string(uri));
    data.extend_from_slice(user.as_ref());
    data.push(is_mayhem_mode as u8);
    data.push(is_cashback_enabled as u8);

    Instruction {
        program_id: PUMP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*mint, true),
            AccountMeta::new_readonly(mint_authority, false),
            AccountMeta::new(bonding_curve, false),
            AccountMeta::new(associated_bonding_curve, false),
            AccountMeta::new_readonly(PUMP_GLOBAL, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(tp, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(MAYHEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(global_params, false),
            AccountMeta::new(sol_vault, false),
            AccountMeta::new(mayhem_state, false),
            AccountMeta::new(mayhem_token_vault, false),
            AccountMeta::new_readonly(EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_PROGRAM_ID, false),
        ],
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_swap_buy_ix(
    user: &Pubkey,
    pool: &Pubkey,
    pool_data: &super::pump_swap::SwapPool,
    base_amount_out: u64,
    max_quote_amount_in: u64,
    base_token_program: &Pubkey,
    quote_token_program: &Pubkey,
    protocol_fee_recipient: &Pubkey,
) -> Instruction {
    let user_base_ata = token_accounts::get_ata(user, &pool_data.base_mint, base_token_program);
    let user_quote_ata = token_accounts::get_ata(user, &pool_data.quote_mint, quote_token_program);
    let (creator_vault_authority, _) = pda::pump_swap_creator_vault_pda(&pool_data.coin_creator);
    let creator_vault_ata = token_accounts::get_ata(
        &creator_vault_authority,
        &pool_data.quote_mint,
        quote_token_program,
    );
    let (global_config, _) = pda::pump_swap_global_config_pda();
    let (user_volume_acc, _) = pda::pump_swap_user_volume_accumulator_pda(user);
    let protocol_fee_ata = token_accounts::get_ata(
        protocol_fee_recipient,
        &pool_data.quote_mint,
        quote_token_program,
    );

    let mut data = Vec::with_capacity(25);
    data.extend_from_slice(&discriminator("buy"));
    data.extend_from_slice(&base_amount_out.to_le_bytes());
    data.extend_from_slice(&max_quote_amount_in.to_le_bytes());
    data.push(0);

    Instruction {
        program_id: PUMP_SWAP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*pool, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(global_config, false),
            AccountMeta::new_readonly(pool_data.base_mint, false),
            AccountMeta::new_readonly(pool_data.quote_mint, false),
            AccountMeta::new(user_base_ata, false),
            AccountMeta::new(user_quote_ata, false),
            AccountMeta::new(pool_data.pool_base_token_account, false),
            AccountMeta::new(pool_data.pool_quote_token_account, false),
            AccountMeta::new_readonly(*protocol_fee_recipient, false),
            AccountMeta::new(protocol_fee_ata, false),
            AccountMeta::new_readonly(*base_token_program, false),
            AccountMeta::new_readonly(*quote_token_program, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(*PUMP_SWAP_EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_SWAP_PROGRAM_ID, false),
            AccountMeta::new(creator_vault_ata, false),
            AccountMeta::new_readonly(creator_vault_authority, false),
            AccountMeta::new_readonly(*PUMP_SWAP_GLOBAL_VOLUME_ACCUMULATOR, false),
            AccountMeta::new(user_volume_acc, false),
            AccountMeta::new_readonly(*PUMP_SWAP_FEE_CONFIG, false),
            AccountMeta::new_readonly(PUMP_FEES_PROGRAM_ID, false),
        ],
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_swap_sell_ix(
    user: &Pubkey,
    pool: &Pubkey,
    pool_data: &super::pump_swap::SwapPool,
    base_amount_in: u64,
    min_quote_amount_out: u64,
    base_token_program: &Pubkey,
    quote_token_program: &Pubkey,
    protocol_fee_recipient: &Pubkey,
) -> Instruction {
    let user_base_ata = token_accounts::get_ata(user, &pool_data.base_mint, base_token_program);
    let user_quote_ata = token_accounts::get_ata(user, &pool_data.quote_mint, quote_token_program);
    let (creator_vault_authority, _) = pda::pump_swap_creator_vault_pda(&pool_data.coin_creator);
    let creator_vault_ata = token_accounts::get_ata(
        &creator_vault_authority,
        &pool_data.quote_mint,
        quote_token_program,
    );
    let (global_config, _) = pda::pump_swap_global_config_pda();
    let protocol_fee_ata = token_accounts::get_ata(
        protocol_fee_recipient,
        &pool_data.quote_mint,
        quote_token_program,
    );

    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(&discriminator("sell"));
    data.extend_from_slice(&base_amount_in.to_le_bytes());
    data.extend_from_slice(&min_quote_amount_out.to_le_bytes());

    Instruction {
        program_id: PUMP_SWAP_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*pool, false),
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(global_config, false),
            AccountMeta::new_readonly(pool_data.base_mint, false),
            AccountMeta::new_readonly(pool_data.quote_mint, false),
            AccountMeta::new(user_base_ata, false),
            AccountMeta::new(user_quote_ata, false),
            AccountMeta::new(pool_data.pool_base_token_account, false),
            AccountMeta::new(pool_data.pool_quote_token_account, false),
            AccountMeta::new_readonly(*protocol_fee_recipient, false),
            AccountMeta::new(protocol_fee_ata, false),
            AccountMeta::new_readonly(*base_token_program, false),
            AccountMeta::new_readonly(*quote_token_program, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(*PUMP_SWAP_EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(PUMP_SWAP_PROGRAM_ID, false),
            AccountMeta::new(creator_vault_ata, false),
            AccountMeta::new_readonly(creator_vault_authority, false),
            AccountMeta::new_readonly(*PUMP_SWAP_FEE_CONFIG, false),
            AccountMeta::new_readonly(PUMP_FEES_PROGRAM_ID, false),
        ],
        data,
    }
}

const METAPLEX_TOKEN_METADATA_ID: Pubkey =
    solana_sdk::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

fn metaplex_metadata_pda(mint: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[
            b"metadata",
            METAPLEX_TOKEN_METADATA_ID.as_ref(),
            mint.as_ref(),
        ],
        &METAPLEX_TOKEN_METADATA_ID,
    );
    pda
}
