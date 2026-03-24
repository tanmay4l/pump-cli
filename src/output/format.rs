use colored::Colorize;
use comfy_table::{ContentArrangement, Table};

pub fn error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

pub fn json_out(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

pub fn table_out(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers.iter().map(|h| h.bold().to_string()));
    for row in rows {
        table.add_row(row);
    }
    println!("{table}");
}

pub fn kv_out(pairs: &[(&str, String)]) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    for (key, value) in pairs {
        table.add_row(vec![key.bold().to_string(), value.clone()]);
    }
    println!("{table}");
}

pub fn format_sol(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    format!("{:.6} SOL", sol)
}

pub fn format_tokens(raw: u64, decimals: u8) -> String {
    let amount = raw as f64 / 10_f64.powi(decimals as i32);
    format!("{:.4}", amount)
}
