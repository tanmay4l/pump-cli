pub mod autosell;
pub mod balance;
pub mod config_cmd;
pub mod create;
pub mod info;
pub mod keys;
pub mod portfolio;
pub mod snipe;
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

    /// Priority fee in microlamports per compute unit
    #[arg(long, global = true)]
    pub priority_fee: Option<u64>,

    /// Send transaction via Jito block engine
    #[arg(long, global = true, default_value = "false")]
    pub jito: bool,

    /// Jito tip in lamports (default: 10000 = 0.00001 SOL)
    #[arg(long, global = true)]
    pub jito_tip: Option<u64>,
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
        /// Amount of tokens to sell (mutually exclusive with --percent and --all)
        #[arg(long, conflicts_with_all = ["percent", "all"])]
        amount: Option<f64>,
        /// Sell a percentage of your holdings (1-100)
        #[arg(long, conflicts_with_all = ["amount", "all"], value_parser = clap::value_parser!(u8).range(1..=100))]
        percent: Option<u8>,
        /// Sell all tokens
        #[arg(long, conflicts_with_all = ["amount", "percent"])]
        all: bool,
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
    /// Snipe new pump.fun tokens — auto-buy on creation
    Snipe {
        /// Amount of SOL to spend per buy
        #[arg(long)]
        amount: f64,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "1000")]
        slippage: u64,
        /// Key name to use for signing (single wallet mode)
        #[arg(long, conflicts_with = "rotate_keys")]
        key: Option<String>,
        /// Rotate through all stored keys (round-robin per buy)
        #[arg(long)]
        rotate_keys: bool,
        /// Minimum SOL in curve to buy (filter out tiny launches)
        #[arg(long)]
        min_sol: Option<f64>,
        /// Maximum SOL in curve to buy (filter out large launches)
        #[arg(long)]
        max_sol: Option<f64>,
        /// Enable rug detection checks before buying
        #[arg(long)]
        rug_check: bool,
        /// Minimum creator SOL balance (default: 0.05)
        #[arg(long)]
        min_creator_sol: Option<f64>,
        /// Minimum creator transaction count (default: 5)
        #[arg(long)]
        min_creator_txns: Option<usize>,
        /// Skip tokens where freeze authority is active
        #[arg(long)]
        reject_freeze: bool,
    },
    /// Auto-sell: watch price and sell on take-profit, stop-loss, or trailing stop
    AutoSell {
        /// Token mint address
        mint: String,
        /// Take profit: sell when price rises this % from entry
        #[arg(long)]
        take_profit: Option<f64>,
        /// Stop loss: sell when price drops this % from entry
        #[arg(long)]
        stop_loss: Option<f64>,
        /// Trailing stop: sell when price drops this % from peak
        #[arg(long)]
        trailing_stop: Option<f64>,
        /// Percentage of holdings to sell when triggered (1-100)
        #[arg(long, default_value = "100", value_parser = clap::value_parser!(u8).range(1..=100))]
        sell_percent: u8,
        /// Slippage tolerance in basis points
        #[arg(long, default_value = "500")]
        slippage: u64,
        /// Price check interval in seconds
        #[arg(long, default_value = "2")]
        interval: u64,
        /// Key name to use for signing
        #[arg(long)]
        key: Option<String>,
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
    let tx_opts = crate::wallet::TxOptions::resolve(cli.priority_fee, cli.jito, cli.jito_tip)?;

    match cli.command {
        Commands::Config { action } => config_cmd::handle(action, &fmt).await,
        Commands::Keys { action } => keys::handle(action, &fmt).await,
        Commands::Buy {
            mint,
            amount,
            slippage,
            key,
        } => trade::handle_buy(&mint, amount, slippage, key.as_deref(), &fmt, &tx_opts).await,
        Commands::Sell {
            mint,
            amount,
            percent,
            all,
            slippage,
            key,
        } => {
            let sell_amount = trade::SellAmount::resolve(amount, percent, all)?;
            trade::handle_sell(&mint, sell_amount, slippage, key.as_deref(), &fmt, &tx_opts).await
        }
        Commands::Info { mint } => info::handle(&mint, &fmt).await,
        Commands::Balance { mint, key, address } => {
            balance::handle(mint.as_deref(), key.as_deref(), address.as_deref(), &fmt).await
        }
        Commands::Create {
            name,
            symbol,
            uri,
            key,
        } => create::handle(&name, &symbol, &uri, key.as_deref(), &fmt, &tx_opts).await,
        Commands::CreateV2 {
            name,
            symbol,
            uri,
            key,
            mayhem,
            cashback,
        } => {
            create::handle_v2(
                &name,
                &symbol,
                &uri,
                key.as_deref(),
                mayhem,
                cashback,
                &fmt,
                &tx_opts,
            )
            .await
        }
        Commands::Swap { action } => swap::handle(action, &fmt, &tx_opts).await,
        Commands::Portfolio { key, address } => {
            portfolio::handle(key.as_deref(), address.as_deref(), &fmt).await
        }
        Commands::Snipe {
            amount,
            slippage,
            key,
            rotate_keys,
            min_sol,
            max_sol,
            rug_check,
            min_creator_sol,
            min_creator_txns,
            reject_freeze,
        } => {
            let rug_cfg = if rug_check
                || min_creator_sol.is_some()
                || min_creator_txns.is_some()
                || reject_freeze
            {
                Some(snipe::SnipeRugConfig {
                    min_creator_sol: min_creator_sol.map(|s| (s * 1e9) as u64),
                    min_creator_txns,
                    reject_freeze,
                })
            } else {
                None
            };
            snipe::handle(snipe::SnipeConfig {
                sol_amount: amount,
                slippage_bps: slippage,
                key_name: key.as_deref(),
                rotate_keys,
                tx_opts: &tx_opts,
                min_sol,
                max_sol,
                rug_cfg,
            })
            .await
        }
        Commands::AutoSell {
            mint,
            take_profit,
            stop_loss,
            trailing_stop,
            sell_percent,
            slippage,
            interval,
            key,
        } => {
            if take_profit.is_none() && stop_loss.is_none() && trailing_stop.is_none() {
                anyhow::bail!(
                    "specify at least one trigger: --take-profit, --stop-loss, or --trailing-stop"
                );
            }
            let cfg = autosell::AutoSellConfig {
                take_profit_pct: take_profit,
                stop_loss_pct: stop_loss,
                trailing_stop_pct: trailing_stop,
                sell_percent,
                slippage_bps: slippage,
                interval_secs: interval,
            };
            autosell::handle(&mint, key.as_deref(), &tx_opts, cfg).await
        }
        Commands::Watch { mint, interval } => watch::handle(&mint, interval).await,
    }
}
