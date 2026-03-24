use anyhow::Context;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::cmd::OutputFormat;
use crate::core::{constants, pda};
use crate::output::{self, format};
use crate::rpc::PumpRpcClient;

pub async fn handle(mint_str: &str, fmt: &OutputFormat) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let client = PumpRpcClient::new()?;

    let (bc_address, _) = pda::bonding_curve_pda(&mint);
    let curve = client.get_bonding_curve(&bc_address)?;

    let price = curve.price_sol();
    let mcap = curve.market_cap_sol();
    let progress = curve.progress() * 100.0;

    output::emit(
        fmt,
        &serde_json::json!({
            "mint": mint_str,
            "creator": curve.creator.to_string(),
            "bonding_curve": bc_address.to_string(),
            "price_sol": price,
            "market_cap_sol": mcap,
            "progress_pct": progress,
            "complete": curve.complete,
            "is_mayhem_mode": curve.is_mayhem_mode,
            "is_cashback_coin": curve.is_cashback_coin,
            "virtual_sol_reserves": curve.virtual_sol_reserves,
            "virtual_token_reserves": curve.virtual_token_reserves,
            "real_sol_reserves": curve.real_sol_reserves,
            "real_token_reserves": curve.real_token_reserves,
            "token_total_supply": curve.token_total_supply,
        }),
        &[
            ("Mint", mint_str.to_string()),
            ("Creator", curve.creator.to_string()),
            ("Bonding Curve", bc_address.to_string()),
            ("Price", format!("{:.10} SOL", price)),
            ("Market Cap", format!("{:.4} SOL", mcap)),
            ("Progress", format!("{:.2}%", progress)),
            (
                "Complete",
                if curve.complete { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Mayhem Mode",
                if curve.is_mayhem_mode { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Cashback Coin",
                if curve.is_cashback_coin { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Virtual SOL",
                format::format_sol(curve.virtual_sol_reserves),
            ),
            (
                "Virtual Tokens",
                format::format_tokens(curve.virtual_token_reserves, constants::TOKEN_DECIMALS),
            ),
            ("Real SOL", format::format_sol(curve.real_sol_reserves)),
            (
                "Real Tokens",
                format::format_tokens(curve.real_token_reserves, constants::TOKEN_DECIMALS),
            ),
        ],
    );

    Ok(())
}
