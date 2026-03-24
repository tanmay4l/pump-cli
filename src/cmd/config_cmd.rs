use clap::Subcommand;
use serde_json::json;

use crate::cmd::OutputFormat;
use crate::config;
use crate::output;

#[derive(Subcommand)]
pub enum ConfigAction {
    /// List current configuration
    List,
    /// Set a configuration value
    Set {
        /// Key to set (rpc_url, active_key, output)
        key: String,
        /// Value to set
        value: String,
    },
}

pub async fn handle(action: ConfigAction, fmt: &OutputFormat) -> anyhow::Result<()> {
    match action {
        ConfigAction::List => {
            let s = config::load()?;
            output::emit(
                fmt,
                &json!({
                    "rpc_url": s.rpc_url,
                    "active_key": s.active_key,
                    "output": s.output,
                    "priority_fee": s.priority_fee,
                    "jito_tip": s.jito_tip,
                }),
                &[
                    ("rpc_url", s.rpc_url.clone()),
                    ("active_key", s.active_key.clone()),
                    ("output", s.output.clone()),
                    (
                        "priority_fee",
                        s.priority_fee
                            .map(|v| format!("{v} microlamports"))
                            .unwrap_or_else(|| "not set".into()),
                    ),
                    (
                        "jito_tip",
                        s.jito_tip
                            .map(|v| format!("{v} lamports"))
                            .unwrap_or_else(|| "not set".into()),
                    ),
                ],
            );
        }
        ConfigAction::Set { key, value } => {
            let mut settings = config::load()?;
            match key.as_str() {
                "rpc_url" => settings.rpc_url = value.clone(),
                "active_key" => settings.active_key = value.clone(),
                "output" => settings.output = value.clone(),
                "priority_fee" => {
                    settings.priority_fee =
                        Some(value.parse().map_err(|_| anyhow::anyhow!("must be a number"))?)
                }
                "jito_tip" => {
                    settings.jito_tip =
                        Some(value.parse().map_err(|_| anyhow::anyhow!("must be a number"))?)
                }
                _ => anyhow::bail!(
                    "unknown config key: {key}. valid: rpc_url, active_key, output, priority_fee, jito_tip"
                ),
            }
            config::save(&settings)?;
            output::emit(
                fmt,
                &json!({ "key": key, "value": value }),
                &[("set", format!("{key} = {value}"))],
            );
        }
    }
    Ok(())
}
