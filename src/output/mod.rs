pub mod format;

pub use format::error;

use crate::cmd::OutputFormat;

pub fn emit(fmt: &OutputFormat, json: &serde_json::Value, kv: &[(&str, String)]) {
    match fmt {
        OutputFormat::Json => format::json_out(json),
        OutputFormat::Table => format::kv_out(kv),
    }
}
