pub mod balance;
pub mod config_cmd;
pub mod create;
pub mod info;
pub mod keys;
pub mod portfolio;
pub mod swap;
pub mod trade;
pub mod watch;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pump", about = "Minimal CLI for Pump.fun on Solana")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: config_cmd::ConfigAction,
    },
    /// Manage keypairs
    Keys {
        #[command(subcommand)]
        action: keys::KeysAction,
    },
    /// Buy tokens on bonding curve
    Buy {
        /// Token mint address
        mint: String,
        /// Amount of SOL to spend
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "500")]
        slippage: u64,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
    },
    /// Sell tokens on bonding curve
    Sell {
        /// Token mint address
        mint: String,
        /// Amount of tokens to sell
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "500")]
        slippage: u64,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
    },
    /// Show bonding curve info for a token
    Info {
        /// Token mint address
        mint: String,
    },
    /// Show SOL or token balance
    Balance {
        /// Token mint address (omit for SOL balance)
        mint: Option<String>,
        /// Key name or address to check
        #[arg(long)]
        key: Option<String>,
        /// Public address to check (read-only)
        #[arg(long)]
        address: Option<String>,
    },
    /// Create a new token on Pump.fun (legacy path)
    Create {
        /// Token name
        #[arg(long)]
        name: String,
        /// Token symbol
        #[arg(long)]
        symbol: String,
        /// Metadata URI (IPFS or HTTP)
        #[arg(long)]
        uri: String,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
    },
    /// Create a new token on Pump.fun (v2 — supports mayhem mode, cashback)
    CreateV2 {
        /// Token name
        #[arg(long)]
        name: String,
        /// Token symbol
        #[arg(long)]
        symbol: String,
        /// Metadata URI (IPFS or HTTP)
        #[arg(long)]
        uri: String,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
        /// Enable mayhem mode
        #[arg(long, default_value = "false")]
        mayhem: bool,
        /// Enable cashback
        #[arg(long, default_value = "false")]
        cashback: bool,
    },
    /// Trade on PumpSwap AMM (post-graduation)
    Swap {
        #[command(subcommand)]
        action: swap::SwapAction,
    },
    /// Show portfolio (all token holdings)
    Portfolio {
        /// Key name to use
        #[arg(long)]
        key: Option<String>,
        /// Public address to check (read-only)
        #[arg(long)]
        address: Option<String>,
    },
    /// Watch live price of a token
    Watch {
        /// Token mint address
        mint: String,
        /// Poll interval in seconds
        #[arg(long, default_value = "2")]
        interval: u64,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let fmt = cli.format.clone();

    match cli.command {
        Commands::Config { action } => config_cmd::handle(action, &fmt).await,
        Commands::Keys { action } => keys::handle(action, &fmt).await,
        Commands::Buy {
            mint,
            amount,
            slippage,
            key,
        } => trade::handle_buy(&mint, amount, slippage, key.as_deref(), &fmt).await,
        Commands::Sell {
            mint,
            amount,
            slippage,
            key,
        } => trade::handle_sell(&mint, amount, slippage, key.as_deref(), &fmt).await,
        Commands::Info { mint } => info::handle(&mint, &fmt).await,
        Commands::Balance { mint, key, address } => {
            balance::handle(mint.as_deref(), key.as_deref(), address.as_deref(), &fmt).await
        }
        Commands::Create {
            name,
            symbol,
            uri,
            key,
        } => create::handle(&name, &symbol, &uri, key.as_deref(), &fmt).await,
        Commands::CreateV2 {
            name,
            symbol,
            uri,
            key,
            mayhem,
            cashback,
        } => create::handle_v2(&name, &symbol, &uri, key.as_deref(), mayhem, cashback, &fmt).await,
        Commands::Swap { action } => swap::handle(action, &fmt).await,
        Commands::Portfolio { key, address } => {
            portfolio::handle(key.as_deref(), address.as_deref(), &fmt).await
        }
        Commands::Watch { mint, interval } => watch::handle(&mint, interval).await,
    }
}
