use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "pump";
const SETTINGS_FILE: &str = "settings.json";
const KEYS_DIR: &str = "keys";

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_rpc")]
    pub rpc_url: String,
    #[serde(default = "default_key")]
    pub active_key: String,
    #[serde(default = "default_output")]
    pub output: String,
}

fn default_rpc() -> String {
    "https://api.mainnet-beta.solana.com".to_string()
}

fn default_key() -> String {
    "default".to_string()
}

fn default_output() -> String {
    "table".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rpc_url: default_rpc(),
            active_key: default_key(),
            output: default_output(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("could not determine config directory")
        .join(CONFIG_DIR)
}

pub fn keys_dir() -> PathBuf {
    config_dir().join(KEYS_DIR)
}

pub fn settings_path() -> PathBuf {
    config_dir().join(SETTINGS_FILE)
}

pub fn init() -> anyhow::Result<()> {
    fs::create_dir_all(keys_dir())?;
    if !settings_path().exists() {
        let settings = Settings::default();
        let json = serde_json::to_string_pretty(&settings)?;
        fs::write(settings_path(), json)?;
    }
    Ok(())
}

pub fn load() -> anyhow::Result<Settings> {
    init()?;
    let data = fs::read_to_string(settings_path())?;
    let settings: Settings = serde_json::from_str(&data)?;
    Ok(settings)
}

pub fn save(settings: &Settings) -> anyhow::Result<()> {
    init()?;
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(settings_path(), json)?;
    Ok(())
}
