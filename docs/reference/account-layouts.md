# Account Layouts

Byte offsets from IDL type definitions. Used by `src/core/global.rs` for parsing.

## Pump Global (741 bytes, IDL type `Global`)

| Offset | Field | Type | Size |
|--------|-------|------|------|
| 0 | discriminator | [u8;8] | 8 |
| 8 | initialized | bool | 1 |
| 9 | authority | Pubkey | 32 |
| 41 | **fee_recipient** | Pubkey | 32 |
| 73 | initial_virtual_token_reserves | u64 | 8 |
| 81 | initial_virtual_sol_reserves | u64 | 8 |
| 89 | initial_real_token_reserves | u64 | 8 |
| 97 | token_total_supply | u64 | 8 |
| 105 | fee_basis_points | u64 | 8 |
| 113 | withdraw_authority | Pubkey | 32 |
| 145 | enable_migrate | bool | 1 |
| 146 | pool_migration_fee | u64 | 8 |
| 154 | creator_fee_basis_points | u64 | 8 |
| 162 | **fee_recipients** | [Pubkey;7] | 224 |
| 386 | set_creator_authority | Pubkey | 32 |
| 418 | admin_set_creator_authority | Pubkey | 32 |
| 450 | create_v2_enabled | bool | 1 |
| 451 | whitelist_pda | Pubkey | 32 |
| 483 | reserved_fee_recipient | Pubkey | 32 |
| 515 | mayhem_mode_enabled | bool | 1 |
| 516 | reserved_fee_recipients | [Pubkey;7] | 224 |
| 740 | is_cashback_enabled | bool | 1 |

## PumpSwap GlobalConfig (643 bytes, IDL type `GlobalConfig`)

| Offset | Field | Type | Size |
|--------|-------|------|------|
| 0 | discriminator | [u8;8] | 8 |
| 8 | admin | Pubkey | 32 |
| 40 | lp_fee_basis_points | u64 | 8 |
| 48 | protocol_fee_basis_points | u64 | 8 |
| 56 | disable_flags | u8 | 1 |
| 57 | **protocol_fee_recipients** | [Pubkey;8] | 256 |
| 313 | coin_creator_fee_basis_points | u64 | 8 |
| 321 | admin_set_coin_creator_authority | Pubkey | 32 |
| 353 | whitelist_pda | Pubkey | 32 |
| 385 | reserved_fee_recipient | Pubkey | 32 |
| 417 | mayhem_mode_enabled | bool | 1 |
| 418 | reserved_fee_recipients | [Pubkey;7] | 224 |
| 642 | is_cashback_enabled | bool | 1 |

Source: pump.json and pump_amm.json IDL type definitions.
