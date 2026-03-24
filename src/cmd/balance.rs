use anyhow::Context;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::str::FromStr;

use crate::cmd::OutputFormat;
use crate::core::{constants, token_accounts};
use crate::output::{self, format};
use crate::rpc::PumpRpcClient;
use crate::wallet;

pub async fn handle(
    mint_str: Option<&str>,
    key_name: Option<&str>,
    address_str: Option<&str>,
    fmt: &OutputFormat,
) -> anyhow::Result<()> {
    let client = PumpRpcClient::new()?;

    let pubkey = if let Some(addr) = address_str {
        Pubkey::from_str(addr).context("invalid address")?
    } else {
        let kp = wallet::keypair::load_active(key_name)?;
        kp.pubkey()
    };

    match mint_str {
        None => {
            let lamports = client.get_sol_balance(&pubkey)?;
            output::emit(
                fmt,
                &serde_json::json!({
                    "address": pubkey.to_string(),
                    "balance_sol": lamports as f64 / constants::LAMPORTS_PER_SOL as f64,
                    "balance_lamports": lamports,
                }),
                &[
                    ("Address", pubkey.to_string()),
                    ("Balance", format::format_sol(lamports)),
                ],
            );
        }
        Some(mint_s) => {
            let mint = Pubkey::from_str(mint_s).context("invalid mint address")?;
            let token_program = client.detect_mint_program(&mint)?;
            let ata = token_accounts::get_ata(&pubkey, &mint, &token_program);
            let raw_balance = client.get_token_balance(&ata)?;

            output::emit(
                fmt,
                &serde_json::json!({
                    "address": pubkey.to_string(),
                    "mint": mint_s,
                    "balance": format::format_tokens(raw_balance, constants::TOKEN_DECIMALS),
                    "balance_raw": raw_balance,
                    "token_program": token_program.to_string(),
                }),
                &[
                    ("Address", pubkey.to_string()),
                    ("Mint", mint_s.to_string()),
                    (
                        "Balance",
                        format::format_tokens(raw_balance, constants::TOKEN_DECIMALS),
                    ),
                ],
            );
        }
    }

    Ok(())
}
