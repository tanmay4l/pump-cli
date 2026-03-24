use anyhow::Context;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::str::FromStr;

use crate::cmd::OutputFormat;
use crate::core::{constants, pda};
use crate::output::format;
use crate::rpc::PumpRpcClient;
use crate::wallet;

pub async fn handle(
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

    let sol_balance = client.get_sol_balance(&pubkey)?;
    let holdings = client.get_token_accounts(&pubkey)?;

    if matches!(fmt, OutputFormat::Json) {
        let tokens: Vec<serde_json::Value> = holdings
            .iter()
            .map(|(mint, amount)| {
                let (bc_address, _) = pda::bonding_curve_pda(mint);
                let status = if client.get_bonding_curve(&bc_address).is_ok() {
                    "bonding_curve"
                } else {
                    "graduated"
                };
                serde_json::json!({
                    "mint": mint.to_string(),
                    "balance": format::format_tokens(*amount, constants::TOKEN_DECIMALS),
                    "balance_raw": amount,
                    "status": status,
                })
            })
            .collect();

        format::json_out(&serde_json::json!({
            "address": pubkey.to_string(),
            "sol_balance": format::format_sol(sol_balance),
            "tokens": tokens,
        }));
    } else {
        println!("Address: {}", pubkey);
        println!("SOL:     {}", format::format_sol(sol_balance));
        println!();

        if holdings.is_empty() {
            println!("No token holdings found.");
        } else {
            let headers = &["Mint", "Balance", "Status"];
            let rows: Vec<Vec<String>> = holdings
                .iter()
                .map(|(mint, amount)| {
                    let (bc_address, _) = pda::bonding_curve_pda(mint);
                    let status = if client.get_bonding_curve(&bc_address).is_ok() {
                        "Bonding Curve".to_string()
                    } else {
                        "Graduated".to_string()
                    };
                    vec![
                        mint.to_string(),
                        format::format_tokens(*amount, constants::TOKEN_DECIMALS),
                        status,
                    ]
                })
                .collect();
            format::table_out(headers, rows);
        }
    }

    Ok(())
}
