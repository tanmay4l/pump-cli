//! On-chain rug detection heuristics using only RPC data.

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

/// Rug check results for a single token.
#[derive(Debug)]
pub struct RugReport {
    #[allow(dead_code)]
    pub mint_authority_active: bool,
    #[allow(dead_code)]
    pub freeze_authority_active: bool,
    pub creator_sol_balance: u64,
    pub creator_tx_count: usize,
    pub warnings: Vec<String>,
}

impl RugReport {
    pub fn is_safe(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Configuration for which checks to run and their thresholds.
pub struct RugCheckConfig {
    pub min_creator_sol: Option<u64>,
    pub min_creator_txns: Option<usize>,
    pub reject_mint_authority: bool,
    pub reject_freeze_authority: bool,
}

impl Default for RugCheckConfig {
    fn default() -> Self {
        Self {
            min_creator_sol: Some(50_000_000), // 0.05 SOL
            min_creator_txns: Some(5),
            reject_mint_authority: false, // pump.fun manages this; usually active during bonding curve
            reject_freeze_authority: true,
        }
    }
}

/// Run rug checks against on-chain data.
pub fn check(
    client: &RpcClient,
    mint: &Pubkey,
    creator: &Pubkey,
    config: &RugCheckConfig,
) -> RugReport {
    let mut warnings = Vec::new();

    // 1. Parse mint account for authority flags
    let (mint_auth, freeze_auth) = match client.get_account(mint) {
        Ok(acc) if acc.data.len() >= 82 => parse_mint_authorities(&acc.data),
        _ => (false, false),
    };

    if mint_auth && config.reject_mint_authority {
        warnings.push("mint authority still active — supply can be inflated".into());
    }
    if freeze_auth && config.reject_freeze_authority {
        warnings.push("freeze authority active — your tokens can be frozen".into());
    }

    // 2. Creator SOL balance
    let creator_balance = client.get_balance(creator).unwrap_or(0);
    if let Some(min) = config.min_creator_sol {
        if creator_balance < min {
            warnings.push(format!(
                "creator balance {:.4} SOL < minimum {:.4} SOL — likely throwaway",
                creator_balance as f64 / 1e9,
                min as f64 / 1e9,
            ));
        }
    }

    // 3. Creator transaction history (quick check — just count recent signatures)
    let creator_txns = client
        .get_signatures_for_address(creator)
        .map(|sigs| sigs.len())
        .unwrap_or(0);

    if let Some(min) = config.min_creator_txns {
        if creator_txns < min {
            warnings.push(format!(
                "creator has only {creator_txns} transactions — likely new wallet"
            ));
        }
    }

    RugReport {
        mint_authority_active: mint_auth,
        freeze_authority_active: freeze_auth,
        creator_sol_balance: creator_balance,
        creator_tx_count: creator_txns,
        warnings,
    }
}

/// Parse mint authority and freeze authority from raw SPL Token mint data.
/// Layout: [0..4] option(u32) + [4..36] mint_authority + ... [46..50] option(u32) + [50..82] freeze_authority
fn parse_mint_authorities(data: &[u8]) -> (bool, bool) {
    let mint_auth =
        data.len() >= 4 && u32::from_le_bytes(data[0..4].try_into().unwrap_or_default()) == 1;
    let freeze_auth =
        data.len() >= 50 && u32::from_le_bytes(data[46..50].try_into().unwrap_or_default()) == 1;
    (mint_auth, freeze_auth)
}
