# pump-cli

Minimal CLI for trading on [Pump.fun](https://pump.fun) and PumpSwap on Solana.

Supports bonding curve buy/sell, PumpSwap AMM trading, token creation (v1 + v2), portfolio tracking, and live price watching.

## Install

```bash
# Build from source (requires Rust 1.75+)
cargo build --release
# Binary at ./target/release/pump-cli
```

## Configuration

```bash
# Show current config
pump-cli config list

# Set custom RPC endpoint
pump-cli config set rpc_url https://your-rpc-url.com
```

Config stored at `~/Library/Application Support/pump/settings.json` (macOS) or equivalent.

## Key Management

```bash
# Generate a new keypair
pump-cli keys generate my-wallet

# Import from private key
pump-cli keys import my-wallet --private-key <base58-key>

# Import from seed phrase
pump-cli keys import my-wallet --seed-phrase "word1 word2 ..."

# List keys
pump-cli keys list

# Set active key
pump-cli keys use my-wallet
```

**Security**: Private keys are stored as JSON files in your OS config directory (`~/Library/Application Support/pump/keys/` on macOS). Protect this directory. Never share private keys. Use a dedicated trading wallet with limited funds.

## Trading

### Bonding Curve (pre-graduation)

```bash
# Buy tokens with SOL
pump-cli buy <MINT> --amount 0.1 --slippage 500

# Sell tokens
pump-cli sell <MINT> --amount 1000.0 --slippage 500

# Check token info (price, progress, creator, mayhem/cashback status)
pump-cli info <MINT>
```

`--slippage` is in basis points (500 = 5%).

### PumpSwap AMM (post-graduation)

```bash
# Buy on PumpSwap (takes mint address, resolves pool automatically)
pump-cli swap buy <MINT> --amount 0.1 --slippage 500

# Sell on PumpSwap
pump-cli swap sell <MINT> --amount 1000.0 --slippage 500

# Pool info
pump-cli swap info <MINT>
```

### Token Creation

```bash
# Legacy create
pump-cli create --name "MyToken" --symbol "MTK" --uri "https://example.com/meta.json"

# Create v2 (supports mayhem mode + cashback)
pump-cli create-v2 --name "MyToken" --symbol "MTK" --uri "https://example.com/meta.json" --mayhem --cashback
```

## Other Commands

```bash
# Check SOL balance
pump-cli balance --address <PUBKEY>

# Check token balance
pump-cli balance --address <PUBKEY> <MINT>

# Portfolio (all token holdings)
pump-cli portfolio --address <PUBKEY>

# Watch live price
pump-cli watch <MINT> --interval 2
```

## Output Formats

All commands support `-f json` for machine-readable output:

```bash
pump-cli -f json info <MINT>
pump-cli -f json balance --address <PUBKEY>
```

## Testing

```bash
# CI-stable: IDL parity tests (no network, instant)
cargo test --test idl_discriminators --test idl_account_counts --test idl_account_wiring --test idl_layout_guards

# Best-effort: mainnet RPC read-only tests (may fail on rate limits)
cargo test --test simulate_mainnet

# Manual: local validator integration tests (requires solana-test-validator + network)
cargo test --test local_validator_integration -- --ignored --nocapture --test-threads=1

# Smoke tests (requires release build + network)
cargo build --release && bash tests/smoke.sh
```

## Contributing

```bash
# Before submitting changes, run the full check suite:
cargo fmt --check
cargo clippy --tests -- -D warnings
cargo test -- --nocapture

# To run the full test suite including local validator tests:
cargo test --test local_validator_integration -- --ignored --nocapture --test-threads=1
```

Local validator tests require `solana-test-validator` (Agave 3.1+) and mainnet RPC access for account cloning. They boot a local validator, clone program binaries and accounts, then run real transaction simulations.

## Known Limitations

- Fee recipients are read dynamically from on-chain Global/GlobalConfig accounts. Falls back to hardcoded default with a warning if parsing fails.
- Token2022 mints are detected and handled for ATA derivation. Not all Token2022 extensions are tested for trading.
- PumpSwap instruction parity is verified against IDL. Full swap simulation requires cloning a real pool.
- Buy/sell commands assume the user's Associated Token Account exists.

## License

MIT — see [LICENSE](LICENSE).
