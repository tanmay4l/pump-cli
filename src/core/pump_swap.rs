use anyhow::{bail, Context};
use solana_sdk::pubkey::Pubkey;

use super::constants::PUMP_SWAP_TOTAL_FEE_BPS;

/// On-chain PumpSwap pool account state (from pump_amm.json IDL).
#[derive(Debug)]
pub struct SwapPool {
    #[allow(dead_code)]
    pub pool_bump: u8,
    #[allow(dead_code)]
    pub index: u16,
    pub creator: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    #[allow(dead_code)]
    pub lp_mint: Pubkey,
    pub pool_base_token_account: Pubkey,
    pub pool_quote_token_account: Pubkey,
    pub lp_supply: u64,
    pub coin_creator: Pubkey,
    pub is_mayhem_mode: bool,
    pub is_cashback_coin: bool,
}

const POOL_DISCRIMINATOR: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];

impl SwapPool {
    const MIN_DATA_LEN: usize = 237; // u8 + u16 + 6*Pubkey + u64 + Pubkey + 2*bool

    pub fn validate_discriminator(account_data: &[u8]) -> anyhow::Result<()> {
        if account_data.len() < 8 {
            bail!("pool account data too short for discriminator");
        }
        if account_data[..8] != POOL_DISCRIMINATOR {
            bail!("invalid pool discriminator");
        }
        Ok(())
    }

    pub fn deserialize(data: &[u8]) -> anyhow::Result<Self> {
        if data.len() < Self::MIN_DATA_LEN {
            bail!(
                "pool data too short: {} bytes (need >= {})",
                data.len(),
                Self::MIN_DATA_LEN
            );
        }

        let pool_bump = data[0];
        let index = u16::from_le_bytes(data[1..3].try_into()?);
        let creator = Pubkey::try_from(&data[3..35]).context("invalid creator")?;
        let base_mint = Pubkey::try_from(&data[35..67]).context("invalid base_mint")?;
        let quote_mint = Pubkey::try_from(&data[67..99]).context("invalid quote_mint")?;
        let lp_mint = Pubkey::try_from(&data[99..131]).context("invalid lp_mint")?;
        let pool_base_token_account =
            Pubkey::try_from(&data[131..163]).context("invalid pool_base_token_account")?;
        let pool_quote_token_account =
            Pubkey::try_from(&data[163..195]).context("invalid pool_quote_token_account")?;
        let lp_supply = u64::from_le_bytes(data[195..203].try_into()?);
        let coin_creator = Pubkey::try_from(&data[203..235]).context("invalid coin_creator")?;
        let is_mayhem_mode = data.len() > 235 && data[235] != 0;
        let is_cashback_coin = data.len() > 236 && data[236] != 0;

        Ok(Self {
            pool_bump,
            index,
            creator,
            base_mint,
            quote_mint,
            lp_mint,
            pool_base_token_account,
            pool_quote_token_account,
            lp_supply,
            coin_creator,
            is_mayhem_mode,
            is_cashback_coin,
        })
    }

    /// Constant product buy: tokens_out for given SOL in.
    pub fn calculate_buy(
        &self,
        base_reserves: u64,
        quote_reserves: u64,
        sol_in: u64,
    ) -> anyhow::Result<(u64, u64)> {
        let fee = sol_in * PUMP_SWAP_TOTAL_FEE_BPS / 10_000;
        let sol_after_fee = sol_in - fee;

        let numerator = (sol_after_fee as u128) * (base_reserves as u128);
        let denominator = (quote_reserves as u128) + (sol_after_fee as u128);
        let tokens_out = (numerator / denominator) as u64;

        Ok((tokens_out, fee))
    }

    /// Constant product sell: SOL out for given tokens in.
    pub fn calculate_sell(
        &self,
        base_reserves: u64,
        quote_reserves: u64,
        tokens_in: u64,
    ) -> anyhow::Result<(u64, u64)> {
        let numerator = (tokens_in as u128) * (quote_reserves as u128);
        let denominator = (base_reserves as u128) + (tokens_in as u128);
        let sol_out_before_fee = (numerator / denominator) as u64;

        let fee = sol_out_before_fee * PUMP_SWAP_TOTAL_FEE_BPS / 10_000;
        let sol_out = sol_out_before_fee - fee;

        Ok((sol_out, fee))
    }
}
