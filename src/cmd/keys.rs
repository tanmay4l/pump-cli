use anyhow::bail;
use clap::Subcommand;
use serde_json::json;
use solana_sdk::signer::Signer;

use crate::cmd::OutputFormat;
use crate::config;
use crate::output;
use crate::wallet;

#[derive(Subcommand)]
pub enum KeysAction {
    /// Generate a new keypair
    Generate {
        /// Name for the key
        #[arg(default_value = "default")]
        name: String,
    },
    /// Import a keypair from base58 private key
    Import {
        /// Name for the key
        name: String,
        /// Base58 encoded private key
        #[arg(long)]
        private_key: Option<String>,
        /// BIP39 seed phrase
        #[arg(long)]
        seed_phrase: Option<String>,
    },
    /// List all stored keys
    List,
    /// Set the active key
    Use {
        /// Key name to set as active
        name: String,
    },
}

pub async fn handle(action: KeysAction, fmt: &OutputFormat) -> anyhow::Result<()> {
    match action {
        KeysAction::Generate { name } => {
            let (mnemonic, kp) = wallet::generate_keypair(&name)?;
            let addr = kp.pubkey().to_string();
            output::emit(
                fmt,
                &json!({ "name": name, "address": addr, "mnemonic": mnemonic }),
                &[
                    ("generated key", name),
                    ("address", addr),
                    ("mnemonic", mnemonic),
                ],
            );
            if !matches!(fmt, OutputFormat::Json) {
                eprintln!("\nSave your mnemonic phrase! It cannot be recovered.");
            }
        }
        KeysAction::Import {
            name,
            private_key,
            seed_phrase,
        } => {
            let kp = match (private_key, seed_phrase) {
                (Some(pk), None) => {
                    let bytes = bs58::decode(&pk).into_vec()?;
                    wallet::import_from_bytes(&name, &bytes)?
                }
                (None, Some(phrase)) => wallet::import_from_seed_phrase(&name, &phrase)?,
                _ => bail!("provide either --private-key or --seed-phrase"),
            };
            let addr = kp.pubkey().to_string();
            output::emit(
                fmt,
                &json!({ "name": name, "address": addr }),
                &[("imported key", name), ("address", addr)],
            );
        }
        KeysAction::List => {
            let keys = wallet::list_keys()?;
            let settings = config::load()?;
            if keys.is_empty() {
                output::emit(
                    fmt,
                    &json!({ "keys": [] }),
                    &[("info", "no keys found. run: pump keys generate".into())],
                );
                return Ok(());
            }
            match fmt {
                OutputFormat::Json => {
                    let entries: Vec<_> = keys
                        .iter()
                        .map(|(name, pubkey)| {
                            json!({
                                "name": name,
                                "address": pubkey,
                                "active": *name == settings.active_key,
                            })
                        })
                        .collect();
                    output::emit(fmt, &json!({ "keys": entries }), &[]);
                }
                OutputFormat::Table => {
                    for (name, pubkey) in &keys {
                        let active = if *name == settings.active_key {
                            " (active)"
                        } else {
                            ""
                        };
                        println!("{name}: {pubkey}{active}");
                    }
                }
            }
        }
        KeysAction::Use { name } => {
            wallet::load_keypair(&name)?;
            let mut settings = config::load()?;
            settings.active_key = name.clone();
            config::save(&settings)?;
            output::emit(
                fmt,
                &json!({ "active_key": name }),
                &[("active key set to", name)],
            );
        }
    }
    Ok(())
}
