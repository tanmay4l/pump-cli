use anyhow::Context;
use colored::Colorize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::core::{constants, pda};
use crate::output::format;
use crate::rpc::PumpRpcClient;

fn format_price_change(current: f64, previous: Option<f64>) -> String {
    match previous {
        Some(prev) if prev > 0.0 => {
            let pct = ((current - prev) / prev) * 100.0;
            if pct > 0.0 {
                format!("+{:.2}%", pct).green().to_string()
            } else if pct < 0.0 {
                format!("{:.2}%", pct).red().to_string()
            } else {
                "0.00%".dimmed().to_string()
            }
        }
        _ => "-".dimmed().to_string(),
    }
}

pub async fn handle(mint_str: &str, interval_secs: u64) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let (bc_address, _) = pda::bonding_curve_pda(&mint);

    println!(
        "{} {} (every {}s, Ctrl+C to stop)",
        "Watching".green().bold(),
        mint_str,
        interval_secs
    );
    println!();

    let mut last_price: Option<f64> = None;

    loop {
        let client = PumpRpcClient::new()?;

        match client.get_bonding_curve(&bc_address) {
            Ok(curve) => {
                let price = curve.price_sol();
                let mcap = curve.market_cap_sol();
                let progress = curve.progress() * 100.0;
                let change = format_price_change(price, last_price);

                println!(
                    "Price: {:.10} SOL  |  MCap: {:.4} SOL  |  Progress: {:.1}%  |  {}",
                    price, mcap, progress, change
                );

                if curve.complete {
                    println!(
                        "{}",
                        "Bonding curve complete! Token graduated to PumpSwap."
                            .yellow()
                            .bold()
                    );
                    watch_pumpswap(&mint, interval_secs).await?;
                    return Ok(());
                }

                last_price = Some(price);
            }
            Err(_) => {
                println!("{}", "Not on bonding curve, checking PumpSwap...".dimmed());
                watch_pumpswap(&mint, interval_secs).await?;
                return Ok(());
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
    }
}

async fn watch_pumpswap(mint: &Pubkey, interval_secs: u64) -> anyhow::Result<()> {
    let (pool_authority, _) = pda::pool_authority_pda(mint);
    let (pool_address, _) =
        pda::pump_swap_pool_pda(0, &pool_authority, mint, &constants::WSOL_MINT);

    println!(
        "{} PumpSwap pool {}",
        "Watching".green().bold(),
        &pool_address.to_string()[..8]
    );

    let mut last_price: Option<f64> = None;

    loop {
        let client = PumpRpcClient::new()?;

        match client.get_swap_pool(&pool_address) {
            Ok(pool) => {
                let base_reserves = client.get_token_balance(&pool.pool_base_token_account)?;
                let quote_reserves = client.get_token_balance(&pool.pool_quote_token_account)?;

                let price = if base_reserves > 0 {
                    (quote_reserves as f64 / constants::LAMPORTS_PER_SOL as f64)
                        / (base_reserves as f64 / 10_f64.powi(constants::TOKEN_DECIMALS as i32))
                } else {
                    0.0
                };

                let change = format_price_change(price, last_price);

                println!(
                    "Price: {:.10} SOL  |  Base: {}  |  Quote: {}  |  {}",
                    price,
                    format::format_tokens(base_reserves, constants::TOKEN_DECIMALS),
                    format::format_sol(quote_reserves),
                    change
                );

                last_price = Some(price);
            }
            Err(e) => {
                println!("{} {}", "Error:".red(), e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
    }
}
