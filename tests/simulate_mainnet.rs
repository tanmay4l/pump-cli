use pump_cli::core::bonding_curve::BondingCurve;
use pump_cli::core::constants;
use pump_cli::core::instructions;
use pump_cli::core::pda;
use pump_cli::core::token_accounts;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

const ACTIVE_MINT: &str = "CGKzVhKKYhP5qNrbnD8gKfPtLipsMQX3YkXhHaTVJGnH";
const PUMP_PROGRAM_STR: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

fn mainnet_client() -> RpcClient {
    RpcClient::new("https://api.mainnet-beta.solana.com")
}

#[derive(Debug, PartialEq)]
enum SimDepth {
    EnvelopeOnly,
    ProgramInvoked,
    ProgramErrorAfterInvoke,
}

fn classify(err: &Option<solana_sdk::transaction::TransactionError>, logs: &[String]) -> SimDepth {
    let logs_joined = logs.join("\n");
    let invoked = logs_joined.contains(PUMP_PROGRAM_STR);
    if !invoked {
        SimDepth::EnvelopeOnly
    } else if err.is_some() {
        SimDepth::ProgramErrorAfterInvoke
    } else {
        SimDepth::ProgramInvoked
    }
}

fn print_sim(
    label: &str,
    depth: &SimDepth,
    err: &Option<solana_sdk::transaction::TransactionError>,
    logs: &[String],
) {
    println!("  [{label}] depth = {depth:?}");
    println!("  [{label}] err = {err:?}");
    println!("  [{label}] logs ({} lines):", logs.len());
    for (i, line) in logs.iter().enumerate().take(20) {
        println!("    [{i:2}] {line}");
    }
}

#[test]
fn simulate_buy_mainnet_envelope() {
    let client = mainnet_client();
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (bc_address, _) = pda::bonding_curve_pda(&mint);

    let account = match client.get_account(&bc_address) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let curve = BondingCurve::deserialize(&account.data[8..]).unwrap();
    if curve.complete {
        eprintln!("SKIP: complete");
        return;
    }

    let tp = token_accounts::detect_token_program(&client, &mint).unwrap_or(spl_token::id());
    let user = Keypair::new();
    let token_amount = curve.tokens_for_sol(10_000_000).unwrap();
    let (sol_cost, _) = curve.calculate_buy_cost(token_amount).unwrap();

    let ix = instructions::build_buy_ix(
        &user.pubkey(),
        &mint,
        token_amount,
        sol_cost + sol_cost / 20,
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );

    let bh = match client.get_latest_blockhash() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);

    let cfg = RpcSimulateTransactionConfig {
        sig_verify: false,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let result = match client.simulate_transaction_with_config(&tx, cfg) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };

    let logs = result.value.logs.unwrap_or_default();
    let err = result.value.err;
    let depth = classify(&err, &logs);
    print_sim("mainnet_buy", &depth, &err, &logs);

    let lj = logs.join("\n");
    assert!(!lj.contains("An account required by the instruction is missing"));
    assert!(!lj.contains("invalid program id"));

    println!("  MAINNET BUY: {depth:?}");
}

#[test]
fn simulate_sell_mainnet_envelope() {
    let client = mainnet_client();
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (bc_address, _) = pda::bonding_curve_pda(&mint);

    let account = match client.get_account(&bc_address) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let curve = BondingCurve::deserialize(&account.data[8..]).unwrap();
    if curve.complete {
        eprintln!("SKIP: complete");
        return;
    }

    let tp = token_accounts::detect_token_program(&client, &mint).unwrap_or(spl_token::id());
    let user = Keypair::new();
    let (sol_output, _) = curve.calculate_sell_output(1_000_000).unwrap();

    let ix = instructions::build_sell_ix(
        &user.pubkey(),
        &mint,
        1_000_000,
        sol_output.saturating_sub(sol_output / 20),
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );

    let bh = match client.get_latest_blockhash() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);

    let cfg = RpcSimulateTransactionConfig {
        sig_verify: false,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let result = match client.simulate_transaction_with_config(&tx, cfg) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };

    let logs = result.value.logs.unwrap_or_default();
    let err = result.value.err;
    let depth = classify(&err, &logs);
    print_sim("mainnet_sell", &depth, &err, &logs);

    let lj = logs.join("\n");
    assert!(!lj.contains("An account required by the instruction is missing"));
    assert!(!lj.contains("invalid program id"));

    println!("  MAINNET SELL: {depth:?}");
}

#[test]
fn simulate_buy_wrong_order_mainnet() {
    let client = mainnet_client();
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (bc_address, _) = pda::bonding_curve_pda(&mint);

    let account = match client.get_account(&bc_address) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let curve = BondingCurve::deserialize(&account.data[8..]).unwrap();
    if curve.complete {
        eprintln!("SKIP: complete");
        return;
    }

    let tp = token_accounts::detect_token_program(&client, &mint).unwrap_or(spl_token::id());
    let user = Keypair::new();
    let token_amount = curve.tokens_for_sol(10_000_000).unwrap();
    let (sol_cost, _) = curve.calculate_buy_cost(token_amount).unwrap();

    let mut ix = instructions::build_buy_ix(
        &user.pubkey(),
        &mint,
        token_amount,
        sol_cost + sol_cost / 20,
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );
    ix.accounts.swap(8, 9); // corrupt

    let bh = match client.get_latest_blockhash() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);

    let cfg = RpcSimulateTransactionConfig {
        sig_verify: false,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let result = match client.simulate_transaction_with_config(&tx, cfg) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };

    assert!(result.value.err.is_some(), "wrong order must error");

    let logs = result.value.logs.unwrap_or_default();
    let err = result.value.err;
    let depth = classify(&err, &logs);
    print_sim("mainnet_wrong_order", &depth, &err, &logs);
    println!("  MAINNET WRONG ORDER: {depth:?}");
}

#[test]
fn bonding_curve_read_and_deserialize() {
    let client = mainnet_client();
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (bc_address, _) = pda::bonding_curve_pda(&mint);

    let account = match client.get_account(&bc_address) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("SKIP: not accessible");
            return;
        }
    };

    BondingCurve::validate_discriminator(&account.data).unwrap();
    let curve = BondingCurve::deserialize(&account.data[8..]).unwrap();

    assert!(curve.token_total_supply > 0);
    assert!(curve.virtual_sol_reserves > 0);
    assert!(curve.virtual_token_reserves > 0);
    let price = curve.price_sol();
    assert!(price > 0.0 && !price.is_nan() && !price.is_infinite());

    println!(
        "  Mint: {ACTIVE_MINT}, Price: {:.10} SOL, Creator: {}",
        price, curve.creator
    );
}

#[test]
fn mint_program_detection() {
    let client = mainnet_client();
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let program = match token_accounts::detect_token_program(&client, &mint) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("SKIP: not accessible");
            return;
        }
    };
    assert!(program == spl_token::id() || program == constants::TOKEN_2022_PROGRAM_ID);
    println!("  Token program: {program}");
}

#[test]
fn negative_wrong_discriminator_rejected() {
    let mut data = vec![0u8; 8 + 75];
    data[..8].copy_from_slice(&[0xFF; 8]);
    assert!(BondingCurve::validate_discriminator(&data).is_err());
}

#[test]
fn negative_short_data_rejected() {
    let mut data = [0u8; 8 + 10];
    data[..8].copy_from_slice(&[23, 183, 248, 55, 96, 216, 172, 96]);
    assert!(BondingCurve::deserialize(&data[8..]).is_err());
}

#[test]
fn mayhem_program_exists_on_chain() {
    let client = mainnet_client();
    match client.get_account(&constants::MAYHEM_PROGRAM_ID) {
        Ok(acc) => {
            assert!(acc.executable, "should be executable");
            println!("  Mayhem program: executable=true, owner={}", acc.owner);
        }
        Err(e) => eprintln!("SKIP: {e}"),
    }
}

#[test]
fn create_v2_pda_derivation_domains_correct() {
    let (gp, _) = pda::global_params_pda();
    let (sv, _) = pda::sol_vault_pda();
    let mint = Pubkey::new_unique();
    let (ms, _) = pda::mayhem_state_pda(&mint);

    let exp_gp = Pubkey::find_program_address(&[b"global-params"], &constants::MAYHEM_PROGRAM_ID).0;
    let exp_sv = Pubkey::find_program_address(&[b"sol-vault"], &constants::MAYHEM_PROGRAM_ID).0;
    let exp_ms = Pubkey::find_program_address(
        &[b"mayhem-state", mint.as_ref()],
        &constants::MAYHEM_PROGRAM_ID,
    )
    .0;

    assert_eq!(gp, exp_gp, "global_params domain");
    assert_eq!(sv, exp_sv, "sol_vault domain");
    assert_eq!(ms, exp_ms, "mayhem_state domain");

    let wrong = Pubkey::find_program_address(&[b"global-params"], &constants::PUMP_PROGRAM_ID).0;
    assert_ne!(gp, wrong, "must NOT derive from PUMP_PROGRAM_ID");

    println!("  global_params: {gp}");
    println!("  sol_vault: {sv}");
}

#[test]
fn pump_global_fee_recipients_readable() {
    let client = mainnet_client();
    match pump_cli::core::global::read_pump_fee_recipients(&client) {
        Ok(parsed) => {
            assert_eq!(parsed.primary, constants::FEE_RECIPIENT);
            let all = parsed.all();
            assert_eq!(all.len(), 8, "primary + 7 array = 8 total");
            assert!(
                all.iter().all(|p| *p != Pubkey::default()),
                "no zero recipients"
            );
            println!("  Pump primary: {}", parsed.primary);
            for (i, r) in parsed.recipients.iter().enumerate() {
                println!("  Pump fee_recipients[{i}]: {r}");
            }
            let s0 = parsed.select(0);
            let s1 = parsed.select(1);
            assert_eq!(s0, parsed.primary, "slot 0 mod 8 = index 0 = primary");
            assert_eq!(
                s1, parsed.recipients[0],
                "slot 1 mod 8 = index 1 = first array entry"
            );
            println!("  select(0) = {s0}, select(1) = {s1}");
        }
        Err(e) => eprintln!("SKIP: {e}"),
    }
}

#[test]
fn swap_global_config_fee_recipients_readable() {
    let client = mainnet_client();
    match pump_cli::core::global::read_swap_fee_recipients(&client) {
        Ok(parsed) => {
            assert_eq!(parsed.recipients[0], constants::PROTOCOL_FEE_RECIPIENT);
            assert!(
                parsed.recipients.iter().all(|p| *p != Pubkey::default()),
                "no zero recipients"
            );
            println!("  PumpSwap protocol_fee_recipients:");
            for (i, r) in parsed.recipients.iter().enumerate() {
                println!("    [{i}] {r}");
            }
            let s0 = parsed.select(0);
            let s8 = parsed.select(8);
            assert_eq!(s0, s8, "select wraps: slot 0 == slot 8");
            println!("  select(0) = {s0}");
        }
        Err(e) => eprintln!("SKIP: {e}"),
    }
}
