use anyhow::Context;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, signature::Keypair,
    signer::Signer, transaction::Transaction,
};

const CU_LIMIT: u32 = 100_000;

pub fn sign_and_send(
    client: &RpcClient,
    payer: &Keypair,
    instructions: Vec<Instruction>,
) -> anyhow::Result<String> {
    let mut ixs = vec![ComputeBudgetInstruction::set_compute_unit_limit(CU_LIMIT)];
    ixs.extend(instructions);

    let recent_blockhash = client
        .get_latest_blockhash()
        .context("failed to get recent blockhash")?;

    let tx =
        Transaction::new_signed_with_payer(&ixs, Some(&payer.pubkey()), &[payer], recent_blockhash);

    let sig = client
        .send_and_confirm_transaction(&tx)
        .context("transaction failed")?;

    Ok(sig.to_string())
}
