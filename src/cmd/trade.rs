use anyhow::Context;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::str::FromStr;

use crate::cmd::OutputFormat;
use crate::core::{constants, global, instructions, pda, token_accounts};
use crate::output::{self, format};
use crate::rpc::PumpRpcClient;
use crate::wallet;
use crate::wallet::TxOptions;

pub enum SellAmount {
    Exact(f64),
    Percent(u8),
    All,
}

impl SellAmount {
    pub fn resolve(amount: Option<f64>, percent: Option<u8>, all: bool) -> anyhow::Result<Self> {
        if all {
            Ok(SellAmount::All)
        } else if let Some(pct) = percent {
            Ok(SellAmount::Percent(pct))
        } else if let Some(amt) = amount {
            Ok(SellAmount::Exact(amt))
        } else {
            anyhow::bail!("specify --amount, --percent, or --all")
        }
    }
}

pub async fn handle_buy(
    mint_str: &str,
    sol_amount: f64,
    slippage_bps: u64,
    key_name: Option<&str>,
    fmt: &OutputFormat,
    tx_opts: &TxOptions,
) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let (bc_address, _) = pda::bonding_curve_pda(&mint);
    let curve = client.get_bonding_curve(&bc_address)?;

    let token_program = client.detect_mint_program(&mint)?;
    let fee_recipient = global::select_pump_fee_recipient(&client.inner);

    let sol_lamports = (sol_amount * constants::LAMPORTS_PER_SOL as f64) as u64;
    let token_amount = curve.tokens_for_sol(sol_lamports)?;

    let (sol_cost, fee) = curve.calculate_buy_cost(token_amount)?;
    let max_sol_cost = sol_cost + (sol_cost * slippage_bps / 10_000);

    let ix = instructions::build_buy_ix(
        &kp.pubkey(),
        &mint,
        token_amount,
        max_sol_cost,
        &curve.creator,
        &token_program,
        &fee_recipient,
    );

    let sig = wallet::sign_and_send(&client.inner, &kp, vec![ix], tx_opts).await?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "mint": mint_str,
            "tokens_bought": format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            "sol_spent": format::format_sol(sol_cost),
            "fee": format::format_sol(fee),
            "token_program": token_program.to_string(),
            "fee_recipient": fee_recipient.to_string(),
            "mode": tx_opts.mode_label(),
            "priority_fee": tx_opts.priority_fee,
        }),
        &[
            ("Signature", sig),
            ("Mint", mint_str.to_string()),
            (
                "Tokens bought",
                format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            ),
            ("SOL spent", format::format_sol(sol_cost)),
            ("Fee", format::format_sol(fee)),
            ("Fee recipient", fee_recipient.to_string()),
            ("Mode", tx_opts.mode_label().to_string()),
        ],
    );

    Ok(())
}

pub async fn handle_sell(
    mint_str: &str,
    sell_amount: SellAmount,
    slippage_bps: u64,
    key_name: Option<&str>,
    fmt: &OutputFormat,
    tx_opts: &TxOptions,
) -> anyhow::Result<()> {
    let mint = Pubkey::from_str(mint_str).context("invalid mint address")?;
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let (bc_address, _) = pda::bonding_curve_pda(&mint);
    let curve = client.get_bonding_curve(&bc_address)?;

    let token_program = client.detect_mint_program(&mint)?;
    let fee_recipient = global::select_pump_fee_recipient(&client.inner);

    let token_amount = match sell_amount {
        SellAmount::Exact(f) => (f * 10_f64.powi(constants::TOKEN_DECIMALS as i32)) as u64,
        SellAmount::Percent(pct) => {
            let ata = token_accounts::get_ata(&kp.pubkey(), &mint, &token_program);
            let balance = client.get_token_balance(&ata)?;
            if balance == 0 {
                anyhow::bail!("no tokens to sell");
            }
            balance * (pct as u64) / 100
        }
        SellAmount::All => {
            let ata = token_accounts::get_ata(&kp.pubkey(), &mint, &token_program);
            let balance = client.get_token_balance(&ata)?;
            if balance == 0 {
                anyhow::bail!("no tokens to sell");
            }
            balance
        }
    };

    let (sol_output, fee) = curve.calculate_sell_output(token_amount)?;
    let min_sol_output = sol_output - (sol_output * slippage_bps / 10_000);

    let ix = instructions::build_sell_ix(
        &kp.pubkey(),
        &mint,
        token_amount,
        min_sol_output,
        &curve.creator,
        &token_program,
        &fee_recipient,
    );

    let sig = wallet::sign_and_send(&client.inner, &kp, vec![ix], tx_opts).await?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "mint": mint_str,
            "tokens_sold": format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            "sol_received": format::format_sol(sol_output),
            "fee": format::format_sol(fee),
            "token_program": token_program.to_string(),
            "fee_recipient": fee_recipient.to_string(),
            "mode": tx_opts.mode_label(),
            "priority_fee": tx_opts.priority_fee,
        }),
        &[
            ("Signature", sig),
            ("Mint", mint_str.to_string()),
            (
                "Tokens sold",
                format::format_tokens(token_amount, constants::TOKEN_DECIMALS),
            ),
            ("SOL received", format::format_sol(sol_output)),
            ("Fee", format::format_sol(fee)),
            ("Fee recipient", fee_recipient.to_string()),
            ("Mode", tx_opts.mode_label().to_string()),
        ],
    );

    Ok(())
}
