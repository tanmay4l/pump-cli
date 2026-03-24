use anyhow::{bail, Context};
use bip39::Mnemonic;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::fs;

use crate::config;

pub fn generate_keypair(name: &str) -> anyhow::Result<(String, Keypair)> {
    let entropy: [u8; 16] = rand::random();
    let mnemonic = Mnemonic::from_entropy(&entropy).context("failed to generate mnemonic")?;
    let seed = mnemonic.to_seed("");
    let kp = keypair_from_seed(&seed[..32]).context("failed to create keypair from seed")?;

    save_keypair(name, &kp)?;
    Ok((mnemonic.to_string(), kp))
}

pub fn import_from_bytes(name: &str, data: &[u8]) -> anyhow::Result<Keypair> {
    let kp = match data.len() {
        64 => Keypair::try_from(data).context("invalid 64-byte keypair")?,
        32 => keypair_from_seed(data).context("invalid 32-byte secret key")?,
        _ => bail!("expected 32 or 64 bytes, got {}", data.len()),
    };
    save_keypair(name, &kp)?;
    Ok(kp)
}

pub fn import_from_seed_phrase(name: &str, phrase: &str) -> anyhow::Result<Keypair> {
    let mnemonic = Mnemonic::parse(phrase).context("invalid mnemonic phrase")?;
    let seed = mnemonic.to_seed("");
    let kp = keypair_from_seed(&seed[..32]).context("failed to derive keypair from seed phrase")?;
    save_keypair(name, &kp)?;
    Ok(kp)
}

pub fn load_keypair(name: &str) -> anyhow::Result<Keypair> {
    let path = config::keys_dir().join(format!("{name}.json"));
    if !path.exists() {
        bail!("key '{name}' not found at {}", path.display());
    }
    let data = fs::read_to_string(&path)?;
    let bytes: Vec<u8> = serde_json::from_str(&data)?;
    Keypair::try_from(bytes.as_slice()).context("corrupted key file")
}

pub fn load_active(name_override: Option<&str>) -> anyhow::Result<Keypair> {
    let settings = config::load()?;
    let name = name_override.unwrap_or(&settings.active_key);
    load_keypair(name)
}

pub fn list_keys() -> anyhow::Result<Vec<(String, String)>> {
    let dir = config::keys_dir();
    fs::create_dir_all(&dir)?;
    let mut keys = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            if let Ok(kp) = load_keypair(&name) {
                keys.push((name, kp.pubkey().to_string()));
            }
        }
    }
    Ok(keys)
}

fn keypair_from_seed(seed: &[u8]) -> anyhow::Result<Keypair> {
    use solana_sdk::signer::SeedDerivable;
    Keypair::from_seed(seed).map_err(|e| anyhow::anyhow!("{}", e))
}

fn save_keypair(name: &str, kp: &Keypair) -> anyhow::Result<()> {
    let dir = config::keys_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.json"));
    let bytes = kp.to_bytes().to_vec();
    let json = serde_json::to_string(&bytes)?;
    fs::write(path, json)?;
    Ok(())
}
