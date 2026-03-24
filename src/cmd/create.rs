use solana_sdk::{signature::Keypair, signer::Signer};

use crate::cmd::OutputFormat;
use crate::core::instructions;
use crate::output;
use crate::rpc::PumpRpcClient;
use crate::wallet;
use crate::wallet::TxOptions;

pub async fn handle(
    name: &str,
    symbol: &str,
    uri: &str,
    key_name: Option<&str>,
    fmt: &OutputFormat,
    tx_opts: &TxOptions,
) -> anyhow::Result<()> {
    eprintln!("Warning: using legacy create path. Consider create-v2 for new tokens.");

    let kp = wallet::keypair::load_active(key_name)?;
    let client = PumpRpcClient::new()?;

    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();

    let ix = instructions::build_create_ix(&kp.pubkey(), &mint_pubkey, name, symbol, uri);

    let sig = wallet::build_and_send(
        &client.inner,
        &kp,
        &[&kp, &mint],
        vec![ix],
        200_000,
        tx_opts,
    )
    .await?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "mint": mint_pubkey.to_string(),
            "name": name,
            "symbol": symbol,
            "uri": uri,
            "version": "v1_legacy",
            "mode": tx_opts.mode_label(),
        }),
        &[
            ("Signature", sig),
            ("Mint", mint_pubkey.to_string()),
            ("Name", name.to_string()),
            ("Symbol", symbol.to_string()),
            ("URI", uri.to_string()),
            ("Version", "v1 (legacy)".to_string()),
            ("Mode", tx_opts.mode_label().to_string()),
        ],
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_v2(
    name: &str,
    symbol: &str,
    uri: &str,
    key_name: Option<&str>,
    is_mayhem_mode: bool,
    is_cashback_enabled: bool,
    fmt: &OutputFormat,
    tx_opts: &TxOptions,
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

    let sig = wallet::build_and_send(
        &client.inner,
        &kp,
        &[&kp, &mint],
        vec![ix],
        300_000,
        tx_opts,
    )
    .await?;

    output::emit(
        fmt,
        &serde_json::json!({
            "signature": sig,
            "mint": mint_pubkey.to_string(),
            "name": name,
            "symbol": symbol,
            "uri": uri,
            "version": "v2",
            "token_program": token_program.to_string(),
            "mayhem_program": crate::core::constants::MAYHEM_PROGRAM_ID.to_string(),
            "is_mayhem_mode": is_mayhem_mode,
            "is_cashback_enabled": is_cashback_enabled,
            "mode": tx_opts.mode_label(),
        }),
        &[
            ("Signature", sig),
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
            ("Mode", tx_opts.mode_label().to_string()),
        ],
    );

    Ok(())
}
