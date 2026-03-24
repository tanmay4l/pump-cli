use anyhow::Context;
use clap::Subcommand;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::str::FromStr;

use crate::cmd::OutputFormat;
use crate::core::{constants, global, instructions, pda};
use crate::output::{self, format};
use crate::rpc::PumpRpcClient;
use crate::wallet;

#[derive(Subcommand)]
pub enum SwapAction {
    /// Buy tokens on PumpSwap AMM (post-graduation)
    Buy {
        /// Token mint address
        mint: String,
        /// Amount of SOL to spend
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "500")]
        slippage: u64,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
    },
    /// Sell tokens on PumpSwap AMM (post-graduation)
    Sell {
        /// Token mint address
        mint: String,
        /// Amount of tokens to sell
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "500")]
        slippage: u64,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
    },
    /// Show PumpSwap pool info
    Info {
        /// Token mint address
        mint: String,
    },
}

pub async fn handle(action: SwapAction, fmt: &OutputFormat) -> anyhow::Result<()> {
    match action {
        SwapAction::Buy {
            mint,
            amount,
            slippage,
            key,
        } => handle_buy(&mint, amount, slippage, key.as_deref(), fmt).await,
        SwapAction::Sell {
            mint,
            amount,
            slippage,
            key,
        } => handle_sell(&mint, amount, slippage, key.as_deref(), fmt).await,
        SwapAction::Info { mint } => handle_info(&mint, fmt).await,
    }
}

async fn handle_buy(
    mint_str: &str,
    sol_amount: f64,
    slippage_bps: u64,
    key_name: Option<&str>,
    fmt: &OutputFormat,
) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let (pool_authority, _) = pda::pool_authority_pda(&mint);
    let (pool_address, _) =
        pda::pump_swap_pool_pda(0, &pool_authority, &mint, &constants::WSOL_MINT);
    let pool = client.get_swap_pool(&pool_address)?;

    let base_token_program = client.detect_mint_program(&pool.base_mint)?;
    let quote_token_program = client.detect_mint_program(&pool.quote_mint)?;

    let protocol_fee_recipient = global::select_swap_fee_recipient(&client.inner);

    let base_reserves = client.get_token_balance(&pool.pool_base_token_account)?;
    let quote_reserves = client.get_token_balance(&pool.pool_quote_token_account)?;

    let sol_lamports = (sol_amount * constants::LAMPORTS_PER_SOL as f64) as u64;
    let (tokens_out, fee) = pool.calculate_buy(base_reserves, quote_reserves, sol_lamports)?;

    let max_sol_in = sol_lamports + (sol_lamports * slippage_bps / 10_000);

    let ix = instructions::build_swap_buy_ix(
        &kp.pubkey(),
        &pool_address,
        &pool,
        tokens_out,
        max_sol_in,
        &base_token_program,
        &quote_token_program,
        &protocol_fee_recipient,
    );

    let sig = wallet::sign_and_send(&client.inner, &kp, vec![ix])?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "pool": pool_address.to_string(),
            "tokens_bought": format::format_tokens(tokens_out, constants::TOKEN_DECIMALS),
            "sol_spent": format::format_sol(sol_lamports),
            "fee": format::format_sol(fee),
            "protocol_fee_recipient": protocol_fee_recipient.to_string(),
        }),
        &[
            ("Signature", sig),
            ("Pool", pool_address.to_string()),
            (
                "Tokens bought",
                format::format_tokens(tokens_out, constants::TOKEN_DECIMALS),
            ),
            ("SOL spent", format::format_sol(sol_lamports)),
            ("Fee", format::format_sol(fee)),
            ("Fee recipient", protocol_fee_recipient.to_string()),
        ],
    );

    Ok(())
}

async fn handle_sell(
    mint_str: &str,
    token_amount_f: f64,
    slippage_bps: u64,
    key_name: Option<&str>,
    fmt: &OutputFormat,
) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let (pool_authority, _) = pda::pool_authority_pda(&mint);
    let (pool_address, _) =
        pda::pump_swap_pool_pda(0, &pool_authority, &mint, &constants::WSOL_MINT);
    let pool = client.get_swap_pool(&pool_address)?;

    let base_token_program = client.detect_mint_program(&pool.base_mint)?;
    let quote_token_program = client.detect_mint_program(&pool.quote_mint)?;

    let protocol_fee_recipient = global::select_swap_fee_recipient(&client.inner);

    let base_reserves = client.get_token_balance(&pool.pool_base_token_account)?;
    let quote_reserves = client.get_token_balance(&pool.pool_quote_token_account)?;

    let token_amount = (token_amount_f * 10_f64.powi(constants::TOKEN_DECIMALS as i32)) as u64;
    let (sol_out, fee) = pool.calculate_sell(base_reserves, quote_reserves, token_amount)?;

    let min_sol_out = sol_out - (sol_out * slippage_bps / 10_000);

    let ix = instructions::build_swap_sell_ix(
        &kp.pubkey(),
        &pool_address,
        &pool,
        token_amount,
        min_sol_out,
        &base_token_program,
        &quote_token_program,
        &protocol_fee_recipient,
    );

    let sig = wallet::sign_and_send(&client.inner, &kp, vec![ix])?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "pool": pool_address.to_string(),
            "tokens_sold": format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            "sol_received": format::format_sol(sol_out),
            "fee": format::format_sol(fee),
            "protocol_fee_recipient": protocol_fee_recipient.to_string(),
        }),
        &[
            ("Signature", sig),
            ("Pool", pool_address.to_string()),
            (
                "Tokens sold",
                format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            ),
            ("SOL received", format::format_sol(sol_out)),
            ("Fee", format::format_sol(fee)),
            ("Fee recipient", protocol_fee_recipient.to_string()),
        ],
    );

    Ok(())
}

async fn handle_info(mint_str: &str, fmt: &OutputFormat) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let client = PumpRpcClient::new()?;

    let (pool_authority, _) = pda::pool_authority_pda(&mint);
    let (pool_address, _) =
        pda::pump_swap_pool_pda(0, &pool_authority, &mint, &constants::WSOL_MINT);
    let pool = client.get_swap_pool(&pool_address)?;

    let base_reserves = client.get_token_balance(&pool.pool_base_token_account)?;
    let quote_reserves = client.get_token_balance(&pool.pool_quote_token_account)?;

    let price = if base_reserves > 0 {
        (quote_reserves as f64 / constants::LAMPORTS_PER_SOL as f64)
            / (base_reserves as f64 / 10_f64.powi(constants::TOKEN_DECIMALS as i32))
    } else {
        0.0
    };

    output::emit(
        fmt,
        &serde_json::json!({
            "mint": mint_str,
            "pool": pool_address.to_string(),
            "creator": pool.creator.to_string(),
            "coin_creator": pool.coin_creator.to_string(),
            "base_reserves": base_reserves,
            "quote_reserves": quote_reserves,
            "price_sol": price,
            "lp_supply": pool.lp_supply,
            "is_mayhem_mode": pool.is_mayhem_mode,
            "is_cashback_coin": pool.is_cashback_coin,
        }),
        &[
            ("Mint", mint_str.to_string()),
            ("Pool", pool_address.to_string()),
            ("Creator", pool.creator.to_string()),
            ("Coin Creator", pool.coin_creator.to_string()),
            (
                "Base Reserves",
                format::format_tokens(base_reserves, constants::TOKEN_DECIMALS),
            ),
            ("Quote Reserves", format::format_sol(quote_reserves)),
            ("Price", format!("{:.10} SOL", price)),
            ("LP Supply", pool.lp_supply.to_string()),
            (
                "Mayhem Mode",
                if pool.is_mayhem_mode { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Cashback Coin",
                if pool.is_cashback_coin { "Yes" } else { "No" }.to_string(),
            ),
        ],
    );

    Ok(())
}
