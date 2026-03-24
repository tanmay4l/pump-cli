use anyhow::{bail, Context};
use solana_sdk::pubkey::Pubkey;

use super::constants::{BONDING_CURVE_FEE_BPS, TOKEN_DECIMALS};

/// On-chain bonding curve account state (from pump.json IDL).
#[derive(Debug)]
pub struct BondingCurve {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
    pub creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
}

const BONDING_CURVE_DISCRIMINATOR: [u8; 8] = [23, 183, 248, 55, 96, 216, 172, 96];

impl BondingCurve {
    const MIN_DATA_LEN: usize = 75; // 5*u64 + bool + Pubkey + 2*bool

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < Self::MIN_DATA_LEN {
            bail!(
                "bonding curve data too short: {} bytes (need >= {})",
                data.len(),
                Self::MIN_DATA_LEN,
            );
        }

        let virtual_token_reserves = u64::from_le_bytes(data[0..8].try_into()?);
        let virtual_sol_reserves = u64::from_le_bytes(data[8..16].try_into()?);
        let real_token_reserves = u64::from_le_bytes(data[16..24].try_into()?);
        let real_sol_reserves = u64::from_le_bytes(data[24..32].try_into()?);
        let token_total_supply = u64::from_le_bytes(data[32..40].try_into()?);
        let complete = data[40] != 0;
        let creator = Pubkey::try_from(&data[41..73]).context("invalid creator pubkey")?;
        let is_mayhem_mode = data.len() > 73 && data[73] != 0;
        let is_cashback_coin = data.len() > 74 && data[74] != 0;

        Ok(Self {
            virtual_token_reserves,
            virtual_sol_reserves,
            real_token_reserves,
            real_sol_reserves,
            token_total_supply,
            complete,
            creator,
            is_mayhem_mode,
            is_cashback_coin,
        })
    }

    pub fn validate_discriminator(account_data: &[u8]) -> anyhow::Result<()> {
        if account_data.len() < 8 {
            bail!("account data too short for discriminator");
        }
        if account_data[..8] != BONDING_CURVE_DISCRIMINATOR {
            bail!("invalid bonding curve discriminator");
        }
        Ok(())
    }

    pub fn price_sol(&self) -> f64 {
        let sol = self.virtual_sol_reserves as f64;
        let tokens = self.virtual_token_reserves as f64 / 10_f64.powi(TOKEN_DECIMALS as i32);
        sol / tokens / 1_000_000_000.0
    }

    pub fn market_cap_sol(&self) -> f64 {
        let price = self.price_sol();
        let supply = self.token_total_supply as f64 / 10_f64.powi(TOKEN_DECIMALS as i32);
        price * supply
    }

    pub fn progress(&self) -> f64 {
        if self.token_total_supply == 0 {
            return 0.0;
        }
        let initial_real = 793_100_000_000_000_u64;
        let sold = initial_real.saturating_sub(self.real_token_reserves);
        sold as f64 / initial_real as f64
    }

    /// Returns (sol_cost_lamports, fee_lamports).
    pub fn calculate_buy_cost(&self, token_amount: u64) -> anyhow::Result<(u64, u64)> {
        if self.complete {
            bail!("bonding curve is complete, trade on PumpSwap instead");
        }
        if token_amount > self.real_token_reserves {
            bail!(
                "insufficient tokens on curve: want {} but only {} available",
                token_amount,
                self.real_token_reserves
            );
        }

        let numerator = (token_amount as u128) * (self.virtual_sol_reserves as u128);
        let denominator = (self.virtual_token_reserves as u128) - (token_amount as u128);
        let sol_cost = (numerator / denominator) as u64 + 1; // +1 for rounding up

        let fee = sol_cost * BONDING_CURVE_FEE_BPS / 10_000;
        Ok((sol_cost, fee))
    }

    /// Returns (sol_output_lamports, fee_lamports).
    pub fn calculate_sell_output(&self, token_amount: u64) -> anyhow::Result<(u64, u64)> {
        if self.complete {
            bail!("bonding curve is complete, trade on PumpSwap instead");
        }

        let numerator = (token_amount as u128) * (self.virtual_sol_reserves as u128);
        let denominator = (self.virtual_token_reserves as u128) + (token_amount as u128);
        let sol_output = (numerator / denominator) as u64;

        let fee = sol_output * BONDING_CURVE_FEE_BPS / 10_000;
        let net_output = sol_output.saturating_sub(fee);
        Ok((net_output, fee))
    }

    pub fn tokens_for_sol(&self, sol_lamports: u64) -> anyhow::Result<u64> {
        if self.complete {
            bail!("bonding curve is complete");
        }

        let fee = sol_lamports * BONDING_CURVE_FEE_BPS / 10_000;
        let sol_after_fee = sol_lamports - fee;
        let numerator = (sol_after_fee as u128) * (self.virtual_token_reserves as u128);
        let denominator = (self.virtual_sol_reserves as u128) + (sol_after_fee as u128);
        let tokens = (numerator / denominator) as u64;

        if tokens > self.real_token_reserves {
            bail!("not enough tokens on curve");
        }
        Ok(tokens)
    }
}
