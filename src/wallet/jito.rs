use anyhow::Context;
use base64::Engine;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;

const BUNDLE_ENDPOINT: &str = "https://mainnet.block-engine.jito.wtf/api/v1/bundles";

const TIP_ACCOUNTS: [Pubkey; 8] = [
    pubkey!("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5"),
    pubkey!("HFqU5x63VTqvQss8hp11i4bPuMJrCPQYjh3HJsmLeBp7"),
    pubkey!("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY"),
    pubkey!("ADaUMid9yfUytqMBgopwjb2DTLSZuaZjaRkJ1h2wTpkW"),
    pubkey!("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh"),
    pubkey!("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt"),
    pubkey!("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
    pubkey!("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT"),
];

pub fn select_tip_account(payer: &Pubkey) -> Pubkey {
    TIP_ACCOUNTS[(payer.to_bytes()[0] as usize) % TIP_ACCOUNTS.len()]
}

/// Send a signed transaction as a Jito bundle. Returns the bundle ID.
pub async fn send_bundle(tx: &Transaction) -> anyhow::Result<String> {
    let serialized = bincode::serialize(tx).context("failed to serialize transaction")?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&serialized);

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sendBundle",
        "params": [[encoded]]
    });

    let resp = reqwest::Client::new()
        .post(BUNDLE_ENDPOINT)
        .json(&payload)
        .send()
        .await
        .context("Jito bundle request failed")?;

    let body: serde_json::Value = resp.json().await.context("invalid Jito response")?;

    if let Some(error) = body.get("error") {
        anyhow::bail!("Jito rejected bundle: {error}");
    }

    body["result"]
        .as_str()
        .context("missing bundle ID in Jito response")
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_account_deterministic() {
        let pk = Pubkey::new_unique();
        assert_eq!(select_tip_account(&pk), select_tip_account(&pk));
    }

    #[test]
    fn tip_account_covers_all_slots() {
        let mut seen = std::collections::HashSet::new();
        for i in 0u8..8 {
            let mut bytes = [0u8; 32];
            bytes[0] = i;
            let pk = Pubkey::new_from_array(bytes);
            seen.insert(select_tip_account(&pk));
        }
        assert_eq!(seen.len(), 8, "all 8 tip accounts must be reachable");
    }
}
