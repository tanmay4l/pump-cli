use pump_cli::core::bonding_curve::BondingCurve;
use pump_cli::core::constants;
use pump_cli::core::instructions;
use pump_cli::core::pda;
use pump_cli::core::token_accounts;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;

const ACTIVE_MINT: &str = "CGKzVhKKYhP5qNrbnD8gKfPtLipsMQX3YkXhHaTVJGnH";
const LOCAL_RPC: &str = "http://127.0.0.1:18899";
const VALIDATOR_PORT: u16 = 18899;

#[derive(Debug, PartialEq)]
enum SimDepth {
    EnvelopeOnly,
    ProgramInvoked,
    ProgramErrorAfterInvoke,
}

fn classify(
    err: &Option<solana_sdk::transaction::TransactionError>,
    logs: &[String],
    pid: &str,
) -> SimDepth {
    let joined = logs.join("\n");
    if !joined.contains(pid) {
        return SimDepth::EnvelopeOnly;
    }
    if err.is_some() {
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
    println!("  [{label}] depth={depth:?} err={err:?}");
    for (i, l) in logs.iter().enumerate().take(20) {
        println!("    [{i:2}] {l}");
    }
}

struct ValidatorHandle {
    child: Child,
    #[allow(dead_code)]
    ledger_dir: tempfile::TempDir,
}
impl Drop for ValidatorHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn boot_pump_validator() -> Option<(ValidatorHandle, RpcClient)> {
    boot_validator_with_clones(
        &[constants::PUMP_PROGRAM_ID, constants::PUMP_FEES_PROGRAM_ID],
        &pump_accounts_to_clone(),
    )
}

fn pump_accounts_to_clone() -> Vec<Pubkey> {
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (bc, _) = pda::bonding_curve_pda(&mint);
    let abc = token_accounts::get_ata(&bc, &mint, &spl_token::id());
    vec![
        constants::PUMP_GLOBAL,
        constants::FEE_RECIPIENT,
        constants::EVENT_AUTHORITY,
        mint,
        bc,
        abc,
        *constants::PUMP_GLOBAL_VOLUME_ACCUMULATOR,
        *constants::PUMP_FEE_CONFIG,
    ]
}

fn boot_validator_with_clones(
    programs: &[Pubkey],
    accounts: &[Pubkey],
) -> Option<(ValidatorHandle, RpcClient)> {
    if !Command::new("solana-test-validator")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        eprintln!("SKIP: solana-test-validator not found");
        return None;
    }
    if std::net::TcpStream::connect(format!("127.0.0.1:{VALIDATOR_PORT}")).is_ok() {
        eprintln!("SKIP: port {VALIDATOR_PORT} in use");
        return None;
    }
    let ledger_dir = tempfile::tempdir().ok()?;
    let mut cmd = Command::new("solana-test-validator");
    cmd.arg("--rpc-port")
        .arg(VALIDATOR_PORT.to_string())
        .arg("--ledger")
        .arg(ledger_dir.path())
        .arg("--quiet")
        .arg("--reset")
        .arg("--url")
        .arg("https://api.mainnet-beta.solana.com");
    for p in programs {
        cmd.arg("--clone-upgradeable-program").arg(p.to_string());
    }
    let prog_set: Vec<String> = programs.iter().map(|p| p.to_string()).collect();
    for a in accounts {
        let s = a.to_string();
        if !prog_set.contains(&s) {
            cmd.arg("--clone").arg(s);
        }
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = cmd.spawn().ok()?;
    let handle = ValidatorHandle { child, ledger_dir };
    let client = RpcClient::new(LOCAL_RPC);
    for i in 0..60 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if client.get_health().is_ok() {
            println!("  Validator ready after {}ms", (i + 1) * 500);
            return Some((handle, client));
        }
    }
    eprintln!("SKIP: validator not healthy in 30s");
    None
}

fn fund(client: &RpcClient, user: &Keypair, lamports: u64) -> bool {
    if client.request_airdrop(&user.pubkey(), lamports).is_err() {
        return false;
    }
    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if client.get_balance(&user.pubkey()).unwrap_or(0) >= lamports {
            return true;
        }
    }
    false
}

fn simulate(
    client: &RpcClient,
    tx: &Transaction,
) -> Option<(
    Option<solana_sdk::transaction::TransactionError>,
    Vec<String>,
)> {
    let cfg = RpcSimulateTransactionConfig {
        sig_verify: false,
        replace_recent_blockhash: true,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };
    for attempt in 0..3 {
        match client.simulate_transaction_with_config(tx, cfg.clone()) {
            Ok(r) => return Some((r.value.err, r.value.logs.unwrap_or_default())),
            Err(e) => {
                eprintln!("  sim attempt {}: {e}", attempt + 1);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    None
}

fn create_ata_ix(funder: &Pubkey, owner: &Pubkey, mint: &Pubkey) -> Instruction {
    let ata = spl_associated_token_account::get_associated_token_address(owner, mint);
    Instruction {
        program_id: spl_associated_token_account::id(),
        accounts: vec![
            AccountMeta::new(*funder, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: vec![],
    }
}

fn load_curve(client: &RpcClient, mint: &Pubkey) -> Option<(BondingCurve, Pubkey)> {
    let (bc_addr, _) = pda::bonding_curve_pda(mint);
    let acc = match client.get_account(&bc_addr) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return None;
        }
    };
    let curve = match BondingCurve::validate_discriminator(&acc.data)
        .and_then(|_| BondingCurve::deserialize(&acc.data[8..]))
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return None;
        }
    };
    if curve.complete {
        eprintln!("SKIP: complete");
        return None;
    }
    let tp = client
        .get_account(mint)
        .map(|a| a.owner)
        .unwrap_or(spl_token::id());
    Some((curve, tp))
}

fn setup_user_with_ata(client: &RpcClient, mint: &Pubkey) -> Option<Keypair> {
    let user = Keypair::new();
    if !fund(client, &user, 10 * LAMPORTS_PER_SOL) {
        eprintln!("SKIP: airdrop");
        return None;
    }
    let ix = create_ata_ix(&user.pubkey(), &user.pubkey(), mint);
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);
    let _ = client.send_and_confirm_transaction(&tx);
    Some(user)
}

fn sim_or_skip(
    client: &RpcClient,
    tx: &Transaction,
) -> Option<(
    Option<solana_sdk::transaction::TransactionError>,
    Vec<String>,
)> {
    match simulate(client, tx) {
        Some(r) => Some(r),
        None => {
            eprintln!("SKIP: simulation RPC failed");
            None
        }
    }
}

#[test]
#[ignore]
fn buy_correct_vs_wrong_order() {
    let (_handle, client) = match boot_pump_validator() {
        Some(v) => v,
        None => return,
    };
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (curve, tp) = match load_curve(&client, &mint) {
        Some(v) => v,
        None => return,
    };
    let user = match setup_user_with_ata(&client, &mint) {
        Some(u) => u,
        None => return,
    };

    let token_amount = curve.tokens_for_sol(100_000_000).unwrap();
    let (sol_cost, _) = curve.calculate_buy_cost(token_amount).unwrap();
    let pid = constants::PUMP_PROGRAM_ID.to_string();

    let ix = instructions::build_buy_ix(
        &user.pubkey(),
        &mint,
        token_amount,
        sol_cost + sol_cost / 20,
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);
    let (err, logs) = match sim_or_skip(&client, &tx) {
        Some(r) => r,
        None => return,
    };
    let depth = classify(&err, &logs, &pid);
    print_sim("correct_buy", &depth, &err, &logs);
    assert_ne!(depth, SimDepth::EnvelopeOnly, "must reach program");

    let mut ix_bad = instructions::build_buy_ix(
        &user.pubkey(),
        &mint,
        token_amount,
        sol_cost + sol_cost / 20,
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );
    ix_bad.accounts.swap(0, 2);
    let bh = client.get_latest_blockhash().unwrap();
    let tx_bad = Transaction::new_signed_with_payer(&[ix_bad], Some(&user.pubkey()), &[&user], bh);
    let (err_bad, logs_bad) = match sim_or_skip(&client, &tx_bad) {
        Some(r) => r,
        None => return,
    };
    let depth_bad = classify(&err_bad, &logs_bad, &pid);
    print_sim("wrong_order", &depth_bad, &err_bad, &logs_bad);
    assert!(err_bad.is_some(), "wrong order must error");

    let differ = format!("{err:?}") != format!("{err_bad:?}");
    println!("  Errors differ: {differ}");
}

#[test]
#[ignore]
fn sell_with_ata() {
    let (_handle, client) = match boot_pump_validator() {
        Some(v) => v,
        None => return,
    };
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let (curve, tp) = match load_curve(&client, &mint) {
        Some(v) => v,
        None => return,
    };
    let user = match setup_user_with_ata(&client, &mint) {
        Some(u) => u,
        None => return,
    };

    let ix = instructions::build_sell_ix(
        &user.pubkey(),
        &mint,
        1_000_000,
        0,
        &curve.creator,
        &tp,
        &constants::FEE_RECIPIENT,
    );
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);
    let (err, logs) = match sim_or_skip(&client, &tx) {
        Some(r) => r,
        None => return,
    };
    let depth = classify(&err, &logs, &constants::PUMP_PROGRAM_ID.to_string());
    print_sim("sell", &depth, &err, &logs);
    assert_ne!(depth, SimDepth::EnvelopeOnly, "sell must reach program");
}

#[test]
#[ignore]
fn token_program_detection() {
    let (_handle, client) = match boot_pump_validator() {
        Some(v) => v,
        None => return,
    };
    let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
    let detected = match token_accounts::detect_token_program(&client, &mint) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("SKIP: {e}");
            return;
        }
    };
    assert_eq!(detected, spl_token::id());
    let user = Pubkey::new_unique();
    let ata_spl = token_accounts::get_ata(&user, &mint, &spl_token::id());
    let ata_2022 = token_accounts::get_ata(&user, &mint, &constants::TOKEN_2022_PROGRAM_ID);
    assert_ne!(ata_spl, ata_2022, "SPL and Token2022 ATAs must differ");
}

#[test]
#[ignore]
fn fee_recipients_from_global() {
    let (_handle, client) = match boot_pump_validator() {
        Some(v) => v,
        None => return,
    };
    match pump_cli::core::global::read_pump_fee_recipients(&client) {
        Ok(parsed) => {
            assert_eq!(parsed.primary, constants::FEE_RECIPIENT);
            let all = parsed.all();
            let selected = pump_cli::core::global::select_pump_fee_recipient(&client);
            assert!(all.contains(&selected), "selected must be from array");

            let mint = Pubkey::from_str(ACTIVE_MINT).unwrap();
            if let Some((curve, tp)) = load_curve(&client, &mint) {
                let ix = instructions::build_buy_ix(
                    &Pubkey::new_unique(),
                    &mint,
                    1_000_000,
                    100_000_000,
                    &curve.creator,
                    &tp,
                    &selected,
                );
                assert_eq!(ix.accounts[1].pubkey, selected);
            }
        }
        Err(e) => eprintln!("SKIP: {e}"),
    }
}

#[test]
#[ignore]
fn pumpswap_invocation() {
    let (global_config, _) = pda::pump_swap_global_config_pda();
    let (_handle, client) = match boot_validator_with_clones(
        &[
            constants::PUMP_SWAP_PROGRAM_ID,
            constants::PUMP_FEES_PROGRAM_ID,
        ],
        &[
            global_config,
            *constants::PUMP_SWAP_EVENT_AUTHORITY,
            *constants::PUMP_SWAP_FEE_CONFIG,
            *constants::PUMP_SWAP_GLOBAL_VOLUME_ACCUMULATOR,
        ],
    ) {
        Some(v) => v,
        None => return,
    };

    let user = Keypair::new();
    if !fund(&client, &user, 5 * LAMPORTS_PER_SOL) {
        eprintln!("SKIP: airdrop");
        return;
    }

    let pool_data = pump_cli::core::pump_swap::SwapPool {
        pool_bump: 255,
        index: 0,
        creator: Pubkey::new_unique(),
        base_mint: Pubkey::new_unique(),
        quote_mint: constants::WSOL_MINT,
        lp_mint: Pubkey::default(),
        pool_base_token_account: Pubkey::new_unique(),
        pool_quote_token_account: Pubkey::new_unique(),
        lp_supply: 0,
        coin_creator: Pubkey::new_unique(),
        is_mayhem_mode: false,
        is_cashback_coin: false,
    };
    let ix = instructions::build_swap_buy_ix(
        &user.pubkey(),
        &Pubkey::new_unique(),
        &pool_data,
        1_000_000,
        100_000_000,
        &spl_token::id(),
        &spl_token::id(),
        &constants::PROTOCOL_FEE_RECIPIENT,
    );
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user.pubkey()), &[&user], bh);
    let (err, logs) = match sim_or_skip(&client, &tx) {
        Some(r) => r,
        None => return,
    };
    let depth = classify(&err, &logs, &constants::PUMP_SWAP_PROGRAM_ID.to_string());
    print_sim("pumpswap", &depth, &err, &logs);
    println!(
        "  PumpSwap invoked: {}",
        logs.join("\n")
            .contains(&constants::PUMP_SWAP_PROGRAM_ID.to_string())
    );
}
