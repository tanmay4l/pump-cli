use anyhow::Context;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::config;
use crate::core::bonding_curve::BondingCurve;
use crate::core::pump_swap::SwapPool;

pub struct PumpRpcClient {
    pub inner: RpcClient,
}

impl PumpRpcClient {
    pub fn new() -> anyhow::Result<Self> {
        let settings = config::load()?;
        Ok(Self {
            inner: RpcClient::new(&settings.rpc_url),
        })
    }

    pub fn get_bonding_curve(&self, address: &Pubkey) -> anyhow::Result<BondingCurve> {
        let account = self
            .inner
            .get_account(address)
            .context("bonding curve account not found")?;

        BondingCurve::validate_discriminator(&account.data)?;
        let data = &account.data[8..];
        BondingCurve::deserialize(data)
    }

    pub fn get_swap_pool(&self, address: &Pubkey) -> anyhow::Result<SwapPool> {
        let account = self
            .inner
            .get_account(address)
            .context("swap pool account not found")?;

        SwapPool::validate_discriminator(&account.data)?;
        let data = &account.data[8..];
        SwapPool::deserialize(data)
    }

    pub fn detect_mint_program(&self, mint: &Pubkey) -> anyhow::Result<Pubkey> {
        crate::core::token_accounts::detect_token_program(&self.inner, mint)
    }

    pub fn get_sol_balance(&self, pubkey: &Pubkey) -> anyhow::Result<u64> {
        self.inner
            .get_balance(pubkey)
            .context("failed to get SOL balance")
    }

    pub fn get_token_balance(&self, ata: &Pubkey) -> anyhow::Result<u64> {
        match self.inner.get_token_account_balance(ata) {
            Ok(balance) => {
                let amount: u64 = balance.amount.parse().unwrap_or(0);
                Ok(amount)
            }
            Err(_) => Ok(0),
        }
    }

    /// Returns (mint, raw_balance) pairs for both SPL Token and Token2022.
    pub fn get_token_accounts(&self, owner: &Pubkey) -> anyhow::Result<Vec<(Pubkey, u64)>> {
        use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
        use solana_client::rpc_filter::{Memcmp, RpcFilterType};
        use solana_sdk::commitment_config::CommitmentConfig;

        let mut holdings = Vec::new();

        let programs = [
            spl_token::id(),
            crate::core::constants::TOKEN_2022_PROGRAM_ID,
        ];

        for program_id in &programs {
            let is_token_2022 = *program_id == crate::core::constants::TOKEN_2022_PROGRAM_ID;
            // Token2022 accounts can exceed 165 bytes (extensions), so skip DataSize filter.
            let mut filters = vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                32,
                owner.to_bytes().to_vec(),
            ))];
            if !is_token_2022 {
                filters.insert(0, RpcFilterType::DataSize(165));
            }

            let config = RpcProgramAccountsConfig {
                filters: Some(filters),
                account_config: RpcAccountInfoConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..Default::default()
                },
                ..Default::default()
            };

            match self
                .inner
                .get_program_accounts_with_config(program_id, config)
            {
                Ok(accounts) => {
                    for (_, account) in accounts {
                        if account.data.len() >= 72 {
                            let mint = Pubkey::try_from(&account.data[0..32]).unwrap_or_default();
                            let amount = u64::from_le_bytes(
                                account.data[64..72].try_into().unwrap_or_default(),
                            );
                            if amount > 0 {
                                holdings.push((mint, amount));
                            }
                        }
                    }
                }
                Err(_) => {
                    // Token2022 query may fail on some RPCs; don't fail entire call
                    continue;
                }
            }
        }

        Ok(holdings)
    }

    #[allow(dead_code)]
    pub fn get_token_balance_any(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
        token_program: &Pubkey,
    ) -> anyhow::Result<u64> {
        let ata = crate::core::token_accounts::get_ata(owner, mint, token_program);
        self.get_token_balance(&ata)
    }
}
