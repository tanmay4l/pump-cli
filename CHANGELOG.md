# Changelog

## v0.1.0

Initial release.

### Features

- **Bonding curve trading**: `buy`, `sell` commands with slippage control
- **PumpSwap AMM trading**: `swap buy`, `swap sell` for graduated tokens
- **Token creation**: `create` (legacy) and `create-v2` (mayhem mode, cashback)
- **Token info**: bonding curve state, price, progress, creator, mayhem/cashback status
- **Balance + portfolio**: SOL and token balances, multi-token portfolio view
- **Live price watch**: real-time price polling with configurable interval
- **Key management**: generate, import (private key or seed phrase), list, switch active key
- **Dynamic fee recipients**: reads Pump Global and PumpSwap GlobalConfig on-chain arrays with slot-based round-robin selection
- **Token2022 awareness**: automatic mint program detection and correct ATA derivation
- **create_v2 IDL parity**: Mayhem program PDAs derived under correct program namespace
- **JSON output**: all commands support `-f json` for scripting

### Testing

- 36 IDL parity tests (discriminators, account counts/order/flags, arg layout, PDA domains, fee recipient wiring)
- 11 mainnet RPC tests (bonding curve reads, simulation envelopes, recipient array parsing)
- 5 local validator integration tests (program invocation depth, differentiating wrong-order, PumpSwap probe)
- 38 CLI smoke tests
