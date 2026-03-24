use anyhow::Context;
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_pubsub_client::pubsub_client::PubsubClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::str::FromStr;

use crate::core::{constants, global, instructions, pda, rug_check};
use crate::output::format;
use crate::wallet;
use crate::wallet::TxOptions;

pub struct SnipeRugConfig {
    pub min_creator_sol: Option<u64>,
    pub min_creator_txns: Option<usize>,
    pub reject_freeze: bool,
}

pub struct SnipeConfig<'a> {
    pub sol_amount: f64,
    pub slippage_bps: u64,
    pub key_name: Option<&'a str>,
    pub rotate_keys: bool,
    pub tx_opts: &'a TxOptions,
    pub min_sol: Option<f64>,
    pub max_sol: Option<f64>,
    pub rug_cfg: Option<SnipeRugConfig>,
}

/// Convert an HTTP RPC URL to its websocket equivalent.
fn rpc_to_ws(rpc_url: &str) -> String {
    rpc_url
        .replace("https://", "wss://")
        .replace("http://", "ws://")
}

/// Extract mint address from Pump program create/create_v2 transaction logs.
/// Looks for the "Program log: " lines that contain the mint pubkey.
/// The Pump program emits a log with the mint address after creation.
fn extract_mint_from_logs(logs: &[String]) -> Option<Pubkey> {
    // Strategy: look for account keys mentioned in logs that could be a mint.
    // Pump create logs typically include the mint address in an event log.
    // We look for base58 pubkeys in log lines that aren't known program IDs.
    let known = [
        constants::PUMP_PROGRAM_ID.to_string(),
        constants::PUMP_FEES_PROGRAM_ID.to_string(),
        "11111111111111111111111111111111".to_string(),
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
        constants::TOKEN_2022_PROGRAM_ID.to_string(),
        spl_associated_token_account::id().to_string(),
        constants::EVENT_AUTHORITY.to_string(),
        constants::PUMP_GLOBAL.to_string(),
    ];

    for log in logs {
        // Pump emits event data as base64 after "Program data: "
        // But the most reliable signal is the InitializeMint log from Token program
        // which mentions the mint address, or we can parse the transaction signature
        // and fetch the full transaction to get account keys.

        // Quick heuristic: look for "Program log: " lines with base58 addresses
        if let Some(rest) = log.strip_prefix("Program log: ") {
            // Try to parse any 32-44 char base58 string as a pubkey
            for word in rest.split_whitespace() {
                let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
                if trimmed.len() >= 32 && trimmed.len() <= 44 {
                    if let Ok(pk) = Pubkey::from_str(trimmed) {
                        if !known.contains(&pk.to_string()) && pk != Pubkey::default() {
                            return Some(pk);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract mint from the transaction by fetching it via RPC.
/// More reliable than log parsing — gets the actual account keys.
fn extract_mint_from_tx(client: &RpcClient, sig_str: &str) -> Option<Pubkey> {
    use solana_client::rpc_config::RpcTransactionConfig;
    use solana_sdk::signature::Signature;

    let sig = Signature::from_str(sig_str).ok()?;
    let config = RpcTransactionConfig {
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };
    let tx = client.get_transaction_with_config(&sig, config).ok()?;

    // Look through account keys for a newly created mint
    if let Some(decoded) = tx.transaction.transaction.decode() {
        let keys = decoded.message.static_account_keys();
        for key in keys {
            if known_program(key) {
                continue;
            }
            if let Ok(acc) = client.get_account(key) {
                if (acc.owner == spl_token::id() || acc.owner == constants::TOKEN_2022_PROGRAM_ID)
                    && acc.data.len() >= 82
                {
                    return Some(*key);
                }
            }
        }
    }
    None
}

fn known_program(pk: &Pubkey) -> bool {
    *pk == constants::PUMP_PROGRAM_ID
        || *pk == constants::PUMP_FEES_PROGRAM_ID
        || *pk == constants::PUMP_GLOBAL
        || *pk == constants::EVENT_AUTHORITY
        || *pk == constants::FEE_RECIPIENT
        || *pk == constants::SYSTEM_PROGRAM_ID
        || *pk == constants::RENT_SYSVAR_ID
        || *pk == spl_token::id()
        || *pk == constants::TOKEN_2022_PROGRAM_ID
        || *pk == spl_associated_token_account::id()
        || *pk == constants::MAYHEM_PROGRAM_ID
}

/// Load wallets: either a single key or all keys for rotation.
fn load_wallets(
    key_name: Option<&str>,
    rotate: bool,
) -> anyhow::Result<Vec<(String, solana_sdk::signature::Keypair)>> {
    if rotate {
        let key_list = wallet::list_keys()?;
        if key_list.is_empty() {
            anyhow::bail!("no keys found for rotation — run `pump keys generate` first");
        }
        let mut wallets = Vec::new();
        for (name, _) in &key_list {
            let kp = wallet::load_keypair(name)?;
            wallets.push((name.clone(), kp));
        }
        Ok(wallets)
    } else {
        let settings = crate::config::load()?;
        let name = key_name.unwrap_or(&settings.active_key).to_string();
        let kp = wallet::load_keypair(&name)?;
        Ok(vec![(name, kp)])
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn handle(cfg: SnipeConfig<'_>) -> anyhow::Result<()> {
    let settings = crate::config::load()?;
    let ws_url = rpc_to_ws(&settings.rpc_url);
    let rpc_client = RpcClient::new(&settings.rpc_url);
    let wallets = load_wallets(cfg.key_name, cfg.rotate_keys)?;
    let tx_opts = cfg.tx_opts;
    let slippage_bps = cfg.slippage_bps;
    let min_sol = cfg.min_sol;
    let max_sol = cfg.max_sol;
    let rug_cfg = cfg.rug_cfg;

    let sol_lamports = (cfg.sol_amount * constants::LAMPORTS_PER_SOL as f64) as u64;

    println!("{} for new pump.fun tokens...", "Sniping".green().bold());
    if wallets.len() == 1 {
        println!("  Wallet:   {}", wallets[0].1.pubkey());
    } else {
        println!(
            "  Wallets:  {} keys (round-robin)",
            wallets.len().to_string().cyan()
        );
        for (name, kp) in &wallets {
            println!("    {} → {}", name, kp.pubkey());
        }
    }
    println!("  Amount:   {} SOL per buy", cfg.sol_amount);
    println!("  Slippage: {} bps", slippage_bps);
    println!("  Mode:     {}", if tx_opts.jito { "Jito" } else { "RPC" });
    if tx_opts.priority_fee > 0 {
        println!("  Priority: {} microlamports/CU", tx_opts.priority_fee);
    }
    if let Some(min) = min_sol {
        println!("  Min curve SOL: {min}");
    }
    if let Some(max) = max_sol {
        println!("  Max curve SOL: {max}");
    }
    if rug_cfg.is_some() {
        println!("  Rug check: {}", "enabled".yellow());
    }
    println!("  {}", "Ctrl+C to stop".dimmed());
    println!();

    // Subscribe to Pump program logs
    let filter = solana_client::rpc_config::RpcTransactionLogsFilter::Mentions(vec![
        constants::PUMP_PROGRAM_ID.to_string(),
    ]);
    let config = solana_client::rpc_config::RpcTransactionLogsConfig {
        commitment: Some(CommitmentConfig::confirmed()),
    };

    let (_sub, receiver) = PubsubClient::logs_subscribe(&ws_url, filter, config)
        .context("failed to connect websocket — check your RPC supports websocket")?;

    println!(
        "{}",
        "Connected to websocket. Watching for new token creates...".green()
    );

    let mut wallet_idx: usize = 0;

    for log_response in receiver {
        let logs = &log_response.value.logs;
        let sig = &log_response.value.signature;

        // Check if this is a create instruction
        let is_create = logs.iter().any(|l| l.contains("Instruction: Create"));

        if !is_create {
            continue;
        }

        // Skip failed transactions
        if log_response.value.err.is_some() {
            continue;
        }

        println!(
            "\n{} New token detected! tx: {}",
            "🎯".to_string().green().bold(),
            &sig[..16]
        );

        // Try to extract mint from logs first, then fall back to tx fetch
        let mint = extract_mint_from_logs(logs).or_else(|| extract_mint_from_tx(&rpc_client, sig));

        let mint = match mint {
            Some(m) => m,
            None => {
                println!("  {} Could not extract mint address", "⚠".yellow());
                continue;
            }
        };

        println!("  Mint: {}", mint.to_string().cyan());

        // Read bonding curve
        let (bc_address, _) = pda::bonding_curve_pda(&mint);

        // Small delay to let the curve initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let curve = match rpc_client.get_account(&bc_address) {
            Ok(acc) => {
                match crate::core::bonding_curve::BondingCurve::validate_discriminator(&acc.data)
                    .and_then(|_| {
                        crate::core::bonding_curve::BondingCurve::deserialize(&acc.data[8..])
                    }) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("  {} Curve not ready: {e}", "⏳".yellow());
                        continue;
                    }
                }
            }
            Err(_) => {
                println!("  {} Curve not found yet", "⏳".yellow());
                continue;
            }
        };

        // Apply filters
        if let Some(min) = min_sol {
            let min_lamports = (min * constants::LAMPORTS_PER_SOL as f64) as u64;
            if curve.real_sol_reserves < min_lamports {
                println!(
                    "  {} Skipped: curve SOL {} < min {}",
                    "⏭".dimmed(),
                    format::format_sol(curve.real_sol_reserves),
                    min
                );
                continue;
            }
        }
        if let Some(max) = max_sol {
            let max_lamports = (max * constants::LAMPORTS_PER_SOL as f64) as u64;
            if curve.real_sol_reserves > max_lamports {
                println!(
                    "  {} Skipped: curve SOL {} > max {}",
                    "⏭".dimmed(),
                    format::format_sol(curve.real_sol_reserves),
                    max
                );
                continue;
            }
        }

        let price = curve.price_sol();
        println!(
            "  Price: {:.10} SOL | MCap: {:.4} SOL",
            price,
            curve.market_cap_sol()
        );

        // Rug check (if enabled)
        if let Some(ref cfg) = rug_cfg {
            let rc = rug_check::RugCheckConfig {
                min_creator_sol: cfg.min_creator_sol.or(Some(50_000_000)),
                min_creator_txns: cfg.min_creator_txns.or(Some(5)),
                reject_mint_authority: false,
                reject_freeze_authority: cfg.reject_freeze,
            };
            let report = rug_check::check(&rpc_client, &mint, &curve.creator, &rc);
            println!(
                "  Creator: {} | bal: {:.4} SOL | txns: {}",
                curve.creator,
                report.creator_sol_balance as f64 / 1e9,
                report.creator_tx_count,
            );
            if !report.is_safe() {
                for w in &report.warnings {
                    println!("  {} {}", "⚠ RUG".red().bold(), w);
                }
                println!("  {} Skipped by rug filter", "⏭".dimmed());
                continue;
            }
            println!("  {} Passed rug checks", "✓".green());
        }

        // Calculate buy
        let token_amount = match curve.tokens_for_sol(sol_lamports) {
            Ok(t) => t,
            Err(e) => {
                println!("  {} Buy calc failed: {e}", "✗".red());
                continue;
            }
        };

        let (sol_cost, _fee) = match curve.calculate_buy_cost(token_amount) {
            Ok(c) => c,
            Err(e) => {
                println!("  {} Cost calc failed: {e}", "✗".red());
                continue;
            }
        };

        let max_sol_cost = sol_cost + (sol_cost * slippage_bps / 10_000);
        let token_program = rpc_client
            .get_account(&mint)
            .map(|a| a.owner)
            .unwrap_or(spl_token::id());

        let fee_recipient = global::select_pump_fee_recipient(&rpc_client);

        // Select wallet (round-robin if rotating)
        let (ref wallet_name, ref kp) = wallets[wallet_idx % wallets.len()];
        wallet_idx += 1;

        let ix = instructions::build_buy_ix(
            &kp.pubkey(),
            &mint,
            token_amount,
            max_sol_cost,
            &curve.creator,
            &token_program,
            &fee_recipient,
        );

        let wallet_label = if wallets.len() > 1 {
            format!(" [{}]", wallet_name)
        } else {
            String::new()
        };

        println!(
            "  {} Buying {} tokens for ~{} SOL...{}",
            "→".green().bold(),
            format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            format::format_sol(sol_cost),
            wallet_label.cyan(),
        );

        match wallet::sign_and_send(&rpc_client, kp, vec![ix], tx_opts).await {
            Ok(buy_sig) => {
                println!(
                    "  {} Bought! sig: {}{}",
                    "✓".green().bold(),
                    buy_sig,
                    wallet_label.cyan()
                );
            }
            Err(e) => {
                println!("  {} Buy failed: {e}{}", "✗".red(), wallet_label);
            }
        }
    }

    println!("Websocket disconnected.");
    Ok(())
}
