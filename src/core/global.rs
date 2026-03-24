//! Parse Pump Global (741 bytes) and PumpSwap GlobalConfig (643 bytes).
//! Layout offsets in docs/reference/account-layouts.md.
//! On layout drift, parsing fails and callers fall back to hardcoded constants.

use anyhow::Context;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use super::constants::{PROTOCOL_FEE_RECIPIENT, PUMP_GLOBAL};

fn read_pubkey(data: &[u8], offset: usize) -> anyhow::Result<Pubkey> {
    Pubkey::try_from(&data[offset..offset + 32])
        .map_err(|e| anyhow::anyhow!("invalid pubkey at offset {offset}: {e}"))
}

fn read_pubkey_array<const N: usize>(data: &[u8], offset: usize) -> anyhow::Result<[Pubkey; N]> {
    let mut arr = [Pubkey::default(); N];
    for (i, slot) in arr.iter_mut().enumerate() {
        *slot = read_pubkey(data, offset + i * 32)?;
    }
    Ok(arr)
}

fn deterministic_index(slot: u64, entropy: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325; // FNV offset basis
    for &b in &slot.to_le_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3); // FNV prime
    }
    for &b in entropy {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn select_from(recipients: &[Pubkey], index: u64, fallback: Pubkey) -> Pubkey {
    let active: Vec<Pubkey> = recipients
        .iter()
        .filter(|p| **p != Pubkey::default())
        .copied()
        .collect();
    if active.is_empty() {
        return fallback;
    }
    active[(index as usize) % active.len()]
}

const GLOBAL_DISC: [u8; 8] = [167, 232, 232, 177, 200, 108, 114, 127];

pub const PUMP_GLOBAL_EXPECTED_LEN: usize = 741;

#[allow(dead_code)]
pub struct PumpGlobal {
    pub fee_recipient: Pubkey,       // offset 41
    pub fee_recipients: [Pubkey; 7], // offset 162
}

#[allow(dead_code)]
impl PumpGlobal {
    pub fn all_recipients(&self) -> Vec<Pubkey> {
        let mut v = Vec::with_capacity(8);
        v.push(self.fee_recipient);
        v.extend_from_slice(&self.fee_recipients);
        v
    }

    pub fn select(&self, index: u64) -> Pubkey {
        select_from(&self.all_recipients(), index, self.fee_recipient)
    }
}

pub fn parse_pump_global(data: &[u8]) -> anyhow::Result<PumpGlobal> {
    if data.len() != PUMP_GLOBAL_EXPECTED_LEN {
        anyhow::bail!(
            "Pump Global layout drift: got {} bytes, expected {}",
            data.len(),
            PUMP_GLOBAL_EXPECTED_LEN
        );
    }
    if data[..8] != GLOBAL_DISC {
        anyhow::bail!(
            "Pump Global discriminator mismatch: got {:?}, expected {:?}",
            &data[..8],
            GLOBAL_DISC
        );
    }

    Ok(PumpGlobal {
        fee_recipient: read_pubkey(data, 41)?,
        fee_recipients: read_pubkey_array::<7>(data, 162)?,
    })
}

pub fn parse_pump_fee_recipients(data: &[u8]) -> anyhow::Result<PumpFeeRecipients> {
    let g = parse_pump_global(data)?;
    Ok(PumpFeeRecipients {
        primary: g.fee_recipient,
        recipients: g.fee_recipients,
    })
}

#[derive(Debug)]
pub struct PumpFeeRecipients {
    pub primary: Pubkey,
    pub recipients: [Pubkey; 7],
}

impl PumpFeeRecipients {
    pub fn all(&self) -> Vec<Pubkey> {
        let mut v = Vec::with_capacity(8);
        v.push(self.primary);
        v.extend_from_slice(&self.recipients);
        v
    }

    pub fn select(&self, index: u64) -> Pubkey {
        select_from(&self.all(), index, self.primary)
    }
}

pub fn read_pump_fee_recipients(client: &RpcClient) -> anyhow::Result<PumpFeeRecipients> {
    let account = client
        .get_account(&PUMP_GLOBAL)
        .context("cannot read Pump Global account")?;
    parse_pump_fee_recipients(&account.data)
}

/// Select a fee recipient from the on-chain Global account.
/// Falls back to the hardcoded constant on parse failure.
pub fn select_pump_fee_recipient(client: &RpcClient) -> Pubkey {
    let parsed = match read_pump_fee_recipients(client) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("warn: Pump Global parse failed, using default: {e}");
            return super::constants::FEE_RECIPIENT;
        }
    };
    let index = selection_index(client);
    parsed.select(index)
}

const SWAP_DISC: [u8; 8] = [149, 8, 156, 202, 160, 252, 176, 217];

pub const SWAP_GLOBAL_CONFIG_EXPECTED_LEN: usize = 643;

#[allow(dead_code)]
pub struct SwapGlobalConfig {
    pub protocol_fee_recipients: [Pubkey; 8], // offset 57
}

#[allow(dead_code)]
impl SwapGlobalConfig {
    pub fn select(&self, index: u64) -> Pubkey {
        select_from(&self.protocol_fee_recipients, index, PROTOCOL_FEE_RECIPIENT)
    }
}

pub fn parse_swap_global_config(data: &[u8]) -> anyhow::Result<SwapGlobalConfig> {
    if data.len() != SWAP_GLOBAL_CONFIG_EXPECTED_LEN {
        anyhow::bail!(
            "PumpSwap GlobalConfig layout drift: got {} bytes, expected {}",
            data.len(),
            SWAP_GLOBAL_CONFIG_EXPECTED_LEN
        );
    }
    if data[..8] != SWAP_DISC {
        anyhow::bail!(
            "PumpSwap GlobalConfig discriminator mismatch: got {:?}, expected {:?}",
            &data[..8],
            SWAP_DISC
        );
    }

    Ok(SwapGlobalConfig {
        protocol_fee_recipients: read_pubkey_array::<8>(data, 57)?,
    })
}

pub fn parse_swap_fee_recipients(data: &[u8]) -> anyhow::Result<SwapFeeRecipients> {
    let g = parse_swap_global_config(data)?;
    Ok(SwapFeeRecipients {
        recipients: g.protocol_fee_recipients,
    })
}

#[derive(Debug)]
pub struct SwapFeeRecipients {
    pub recipients: [Pubkey; 8],
}

impl SwapFeeRecipients {
    pub fn select(&self, index: u64) -> Pubkey {
        select_from(&self.recipients, index, PROTOCOL_FEE_RECIPIENT)
    }
}

pub fn read_swap_fee_recipients(client: &RpcClient) -> anyhow::Result<SwapFeeRecipients> {
    let (global_config, _) = super::pda::pump_swap_global_config_pda();
    let account = client
        .get_account(&global_config)
        .context("cannot read PumpSwap GlobalConfig account")?;
    parse_swap_fee_recipients(&account.data)
}

pub fn select_swap_fee_recipient(client: &RpcClient) -> Pubkey {
    let parsed = match read_swap_fee_recipients(client) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("warn: PumpSwap GlobalConfig parse failed, using default: {e}");
            return PROTOCOL_FEE_RECIPIENT;
        }
    };
    let index = selection_index(client);
    parsed.select(index)
}

fn selection_index(client: &RpcClient) -> u64 {
    let slot = client.get_slot().unwrap_or(0);
    let entropy = client
        .get_latest_blockhash()
        .map(|h| h.to_bytes().to_vec())
        .unwrap_or_default();
    deterministic_index(slot, &entropy)
}

#[allow(dead_code)]
pub fn compute_deterministic_index(slot: u64, entropy: &[u8]) -> u64 {
    deterministic_index(slot, entropy)
}

#[allow(dead_code)]
pub fn expected_pump_global_len() -> usize {
    PUMP_GLOBAL_EXPECTED_LEN
}

#[allow(dead_code)]
pub fn expected_swap_global_config_len() -> usize {
    SWAP_GLOBAL_CONFIG_EXPECTED_LEN
}
