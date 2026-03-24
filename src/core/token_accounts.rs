use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

use super::constants::TOKEN_2022_PROGRAM_ID;

pub fn get_ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, token_program)
}

#[allow(dead_code)]
pub fn is_token_2022(program_id: &Pubkey) -> bool {
    *program_id == TOKEN_2022_PROGRAM_ID
}

pub fn detect_token_program(
    client: &solana_client::rpc_client::RpcClient,
    mint: &Pubkey,
) -> anyhow::Result<Pubkey> {
    let account = client
        .get_account(mint)
        .map_err(|_| anyhow::anyhow!("mint account not found: {}", mint))?;
    Ok(account.owner)
}
