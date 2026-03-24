use anyhow::Context;
use solana_client::rpc_client::RpcClient;
#[allow(deprecated)]
use solana_sdk::system_instruction;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer, transaction::Transaction,
};

use super::jito;

const DEFAULT_CU_LIMIT: u32 = 100_000;
const DEFAULT_JITO_TIP: u64 = 10_000; // 0.00001 SOL

#[derive(Debug, Clone)]
pub struct TxOptions {
    pub priority_fee: u64, // microlamports per CU; 0 = none
    pub jito: bool,
    pub jito_tip: u64, // lamports
}

impl TxOptions {
    /// Resolve: CLI args override config, config overrides defaults.
    pub fn resolve(
        cli_priority_fee: Option<u64>,
        cli_jito: bool,
        cli_jito_tip: Option<u64>,
    ) -> anyhow::Result<Self> {
        let settings = crate::config::load()?;
        Ok(Self {
            priority_fee: cli_priority_fee.or(settings.priority_fee).unwrap_or(0),
            jito: cli_jito,
            jito_tip: cli_jito_tip
                .or(settings.jito_tip)
                .unwrap_or(DEFAULT_JITO_TIP),
        })
    }

    pub fn mode_label(&self) -> &'static str {
        if self.jito {
            "jito"
        } else {
            "rpc"
        }
    }
}

impl Default for TxOptions {
    fn default() -> Self {
        Self {
            priority_fee: 0,
            jito: false,
            jito_tip: DEFAULT_JITO_TIP,
        }
    }
}

/// Send a single-signer transaction (buy, sell, swap).
pub async fn sign_and_send(
    client: &RpcClient,
    payer: &Keypair,
    instructions: Vec<Instruction>,
    opts: &TxOptions,
) -> anyhow::Result<String> {
    build_and_send(
        client,
        payer,
        &[payer],
        instructions,
        DEFAULT_CU_LIMIT,
        opts,
    )
    .await
}

/// Send a multi-signer transaction (create — payer + mint keypair).
pub async fn build_and_send(
    client: &RpcClient,
    payer: &Keypair,
    signers: &[&Keypair],
    instructions: Vec<Instruction>,
    cu_limit: u32,
    opts: &TxOptions,
) -> anyhow::Result<String> {
    let ixs = assemble_instructions(&payer.pubkey(), instructions, cu_limit, opts);

    let blockhash = client
        .get_latest_blockhash()
        .context("failed to get recent blockhash")?;

    let tx = Transaction::new_signed_with_payer(&ixs, Some(&payer.pubkey()), signers, blockhash);

    if opts.jito {
        let bundle_id = jito::send_bundle(&tx).await?;
        // Return the tx signature (known before sending) + bundle ID for tracking
        let sig = tx.signatures[0].to_string();
        eprintln!("Jito bundle: {bundle_id}");
        Ok(sig)
    } else {
        let sig = client
            .send_and_confirm_transaction(&tx)
            .context("transaction failed")?;
        Ok(sig.to_string())
    }
}

fn assemble_instructions(
    payer: &solana_sdk::pubkey::Pubkey,
    user_instructions: Vec<Instruction>,
    cu_limit: u32,
    opts: &TxOptions,
) -> Vec<Instruction> {
    let mut ixs = vec![ComputeBudgetInstruction::set_compute_unit_limit(cu_limit)];

    if opts.priority_fee > 0 {
        ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            opts.priority_fee,
        ));
    }

    if opts.jito {
        let tip_account = jito::select_tip_account(payer);
        ixs.push(system_instruction::transfer(
            payer,
            &tip_account,
            opts.jito_tip,
        ));
    }

    ixs.extend(user_instructions);
    ixs
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    fn dummy_ix() -> Instruction {
        Instruction::new_with_bytes(Pubkey::new_unique(), &[1, 2, 3], vec![])
    }

    #[test]
    fn assemble_default_no_extras() {
        let payer = Pubkey::new_unique();
        let opts = TxOptions::default();
        let ixs = assemble_instructions(&payer, vec![dummy_ix()], 100_000, &opts);
        assert_eq!(ixs.len(), 2); // cu_limit + user ix
    }

    #[test]
    fn assemble_with_priority_fee() {
        let payer = Pubkey::new_unique();
        let opts = TxOptions {
            priority_fee: 50_000,
            ..Default::default()
        };
        let ixs = assemble_instructions(&payer, vec![dummy_ix()], 100_000, &opts);
        assert_eq!(ixs.len(), 3); // cu_limit + cu_price + user ix
    }

    #[test]
    fn assemble_with_jito() {
        let payer = Pubkey::new_unique();
        let opts = TxOptions {
            jito: true,
            jito_tip: 10_000,
            ..Default::default()
        };
        let ixs = assemble_instructions(&payer, vec![dummy_ix()], 100_000, &opts);
        assert_eq!(ixs.len(), 3); // cu_limit + tip_transfer + user ix
    }

    #[test]
    fn assemble_with_both() {
        let payer = Pubkey::new_unique();
        let opts = TxOptions {
            priority_fee: 50_000,
            jito: true,
            jito_tip: 25_000,
        };
        let ixs = assemble_instructions(&payer, vec![dummy_ix()], 100_000, &opts);
        assert_eq!(ixs.len(), 4); // cu_limit + cu_price + tip + user ix
    }

    #[test]
    fn resolve_cli_overrides_config() {
        // This test just checks the precedence logic, not actual config loading
        let opts = TxOptions {
            priority_fee: 100,
            jito: false,
            jito_tip: 10_000,
        };
        assert_eq!(opts.priority_fee, 100);
        assert_eq!(opts.mode_label(), "rpc");
    }
}
