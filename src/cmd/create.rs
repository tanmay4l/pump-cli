use solana_sdk::{signature::Keypair, signer::Signer};

use crate::cmd::OutputFormat;
use crate::core::instructions;
use crate::output;
use crate::rpc::PumpRpcClient;
use crate::wallet;

pub async fn handle(
    name: &str,
    symbol: &str,
    uri: &str,
    key_name: Option<&str>,
    fmt: &OutputFormat,
) -> anyhow::Result<()> {
    eprintln!("Warning: using legacy create path. Consider --v2 for new token creation.");

    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();

    let ix = instructions::build_create_ix(&kp.pubkey(), &mint_pubkey, name, symbol, uri);

    let recent_blockhash = client.inner.get_latest_blockhash()?;
    let ixs = vec![
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(200_000),
        ix,
    ];
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &ixs,
        Some(&kp.pubkey()),
        &[&kp, &mint],
        recent_blockhash,
    );

    let sig = client
        .inner
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow::anyhow!("transaction failed: {}", e))?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig.to_string(),
            "mint": mint_pubkey.to_string(),
            "name": name,
            "symbol": symbol,
            "uri": uri,
            "version": "v1_legacy",
        }),
        &[
            ("Signature", sig.to_string()),
            ("Mint", mint_pubkey.to_string()),
            ("Name", name.to_string()),
            ("Symbol", symbol.to_string()),
            ("URI", uri.to_string()),
            ("Version", "v1 (legacy)".to_string()),
        ],
    );

    Ok(())
}

pub async fn handle_v2(
    name: &str,
    symbol: &str,
    uri: &str,
    key_name: Option<&str>,
    is_mayhem_mode: bool,
    is_cashback_enabled: bool,
    fmt: &OutputFormat,
) -> anyhow::Result<()> {
    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();

    let token_program = crate::core::constants::TOKEN_2022_PROGRAM_ID;

    let ix = instructions::build_create_v2_ix(
        &kp.pubkey(),
        &mint_pubkey,
        name,
        symbol,
        uri,
        &token_program,
        is_mayhem_mode,
        is_cashback_enabled,
    );

    let recent_blockhash = client.inner.get_latest_blockhash()?;
    let ixs = vec![
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(300_000),
        ix,
    ];
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &ixs,
        Some(&kp.pubkey()),
        &[&kp, &mint],
        recent_blockhash,
    );

    let sig = client
        .inner
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow::anyhow!("transaction failed: {}", e))?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig.to_string(),
            "mint": mint_pubkey.to_string(),
            "name": name,
            "symbol": symbol,
            "uri": uri,
            "version": "v2",
            "token_program": token_program.to_string(),
            "mayhem_program": crate::core::constants::MAYHEM_PROGRAM_ID.to_string(),
            "is_mayhem_mode": is_mayhem_mode,
            "is_cashback_enabled": is_cashback_enabled,
        }),
        &[
            ("Signature", sig.to_string()),
            ("Mint", mint_pubkey.to_string()),
            ("Name", name.to_string()),
            ("Symbol", symbol.to_string()),
            ("URI", uri.to_string()),
            ("Version", "v2".to_string()),
            ("Token Program", token_program.to_string()),
            (
                "Mayhem Program",
                crate::core::constants::MAYHEM_PROGRAM_ID.to_string(),
            ),
            (
                "Mayhem Mode",
                if is_mayhem_mode { "Yes" } else { "No" }.to_string(),
            ),
            (
                "Cashback",
                if is_cashback_enabled { "Yes" } else { "No" }.to_string(),
            ),
        ],
    );

    Ok(())
}
