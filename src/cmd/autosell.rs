use anyhow::Context;
use colored::Colorize;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::str::FromStr;

use crate::core::{constants, global, instructions, pda, token_accounts};
use crate::output::format;
use crate::rpc::PumpRpcClient;
use crate::wallet;
use crate::wallet::TxOptions;

pub struct AutoSellConfig {
    pub take_profit_pct: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub trailing_stop_pct: Option<f64>,
    pub sell_percent: u8,
    pub slippage_bps: u64,
    pub interval_secs: u64,
}

pub async fn handle(
    mint_str: &str,
    key_name: Option<&str>,
    tx_opts: &TxOptions,
    config: AutoSellConfig,
) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let token_program = client.detect_mint_program(&mint)?;
    let ata = token_accounts::get_ata(&kp.pubkey(), &mint, &token_program);
    let balance = client.get_token_balance(&ata)?;
    if balance == 0 {
        anyhow::bail!("no tokens to sell for mint {mint_str}");
    }

    // Get entry price from current curve state
    let (bc_address, _) = pda::bonding_curve_pda(&mint);
    let curve = client.get_bonding_curve(&bc_address)?;
    let entry_price = curve.price_sol();

    println!("{}", "Auto-sell monitor started".green().bold());
    println!("  Mint:     {mint_str}");
    println!("  Wallet:   {}", kp.pubkey());
    println!(
        "  Balance:  {} tokens",
        format::format_tokens(balance, constants::TOKEN_DECIMALS)
    );
    println!("  Entry:    {:.10} SOL", entry_price);
    println!("  Sell pct: {}%", config.sell_percent);
    if let Some(tp) = config.take_profit_pct {
        println!("  TP:       +{tp}%");
    }
    if let Some(sl) = config.stop_loss_pct {
        println!("  SL:       -{sl}%");
    }
    if let Some(ts) = config.trailing_stop_pct {
        println!("  Trailing: {ts}% from peak");
    }
    println!("  Mode:     {}", tx_opts.mode_label());
    println!("  {}", "Ctrl+C to stop".dimmed());
    println!();

    let mut peak_price = entry_price;
    let mut tick = 0u64;

    loop {
        let client = PumpRpcClient::new()?;

        let current_price = match client.get_bonding_curve(&bc_address) {
            Ok(c) => {
                if c.complete {
                    println!(
                        "  {} Curve graduated — switching to PumpSwap price not yet supported",
                        "⚠".yellow()
                    );
                    println!("  Run `pump-cli swap sell` manually.");
                    return Ok(());
                }
                c.price_sol()
            }
            Err(e) => {
                if tick > 0 {
                    eprintln!("  RPC error: {e}");
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(config.interval_secs)).await;
                tick += 1;
                continue;
            }
        };

        if current_price > peak_price {
            peak_price = current_price;
        }

        let change_from_entry = ((current_price - entry_price) / entry_price) * 100.0;
        let change_from_peak = ((current_price - peak_price) / peak_price) * 100.0;

        let change_str = if change_from_entry >= 0.0 {
            format!("+{:.2}%", change_from_entry).green().to_string()
        } else {
            format!("{:.2}%", change_from_entry).red().to_string()
        };

        print!(
            "\r  Price: {:.10} SOL | {} from entry | peak: {:.10} SOL  ",
            current_price, change_str, peak_price
        );

        // Check take-profit
        if let Some(tp) = config.take_profit_pct {
            if change_from_entry >= tp {
                println!();
                println!(
                    "  {} Take profit triggered at {:.2}% (target: +{tp}%)",
                    "🎯".green().bold(),
                    change_from_entry
                );
                execute_sell(
                    &client,
                    &kp,
                    &mint,
                    &token_program,
                    &bc_address,
                    config.sell_percent,
                    config.slippage_bps,
                    tx_opts,
                )
                .await?;
                return Ok(());
            }
        }

        // Check stop-loss
        if let Some(sl) = config.stop_loss_pct {
            if change_from_entry <= -sl {
                println!();
                println!(
                    "  {} Stop loss triggered at {:.2}% (limit: -{sl}%)",
                    "🛑".red().bold(),
                    change_from_entry
                );
                execute_sell(
                    &client,
                    &kp,
                    &mint,
                    &token_program,
                    &bc_address,
                    config.sell_percent,
                    config.slippage_bps,
                    tx_opts,
                )
                .await?;
                return Ok(());
            }
        }

        // Check trailing stop
        if let Some(ts) = config.trailing_stop_pct {
            if change_from_peak <= -ts && peak_price > entry_price {
                println!();
                println!(
                    "  {} Trailing stop triggered at {:.2}% from peak (limit: -{ts}%)",
                    "📉".yellow().bold(),
                    change_from_peak
                );
                execute_sell(
                    &client,
                    &kp,
                    &mint,
                    &token_program,
                    &bc_address,
                    config.sell_percent,
                    config.slippage_bps,
                    tx_opts,
                )
                .await?;
                return Ok(());
            }
        }

        tick += 1;
        tokio::time::sleep(tokio::time::Duration::from_secs(config.interval_secs)).await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn execute_sell(
    client: &PumpRpcClient,
    kp: &solana_sdk::signature::Keypair,
    mint: &Pubkey,
    token_program: &Pubkey,
    bc_address: &Pubkey,
    sell_percent: u8,
    slippage_bps: u64,
    tx_opts: &TxOptions,
) -> anyhow::Result<()> {
    let ata = token_accounts::get_ata(&kp.pubkey(), mint, token_program);
    let balance = client.get_token_balance(&ata)?;
    if balance == 0 {
        println!("  No tokens remaining.");
        return Ok(());
    }

    let sell_amount = if sell_percent >= 100 {
        balance
    } else {
        balance * (sell_percent as u64) / 100
    };

    let curve = client.get_bonding_curve(bc_address)?;
    let (sol_output, fee) = curve.calculate_sell_output(sell_amount)?;
    let min_sol = sol_output - (sol_output * slippage_bps / 10_000);

    let fee_recipient = global::select_pump_fee_recipient(&client.inner);

    let ix = instructions::build_sell_ix(
        &kp.pubkey(),
        mint,
        sell_amount,
        min_sol,
        &curve.creator,
        token_program,
        &fee_recipient,
    );

    println!(
        "  Selling {} tokens for ~{} SOL...",
        format::format_tokens(sell_amount, constants::TOKEN_DECIMALS),
        format::format_sol(sol_output),
    );

    match wallet::sign_and_send(&client.inner, kp, vec![ix], tx_opts).await {
        Ok(sig) => {
            println!(
                "  {} Sold! SOL received: ~{} (fee: {})",
                "✓".green().bold(),
                format::format_sol(sol_output),
                format::format_sol(fee),
            );
            println!("  sig: {sig}");
        }
        Err(e) => {
            println!("  {} Sell failed: {e}", "✗".red());
        }
    }

    Ok(())
}
