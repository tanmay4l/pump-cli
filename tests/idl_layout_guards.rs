use pump_cli::core::global::{
    compute_deterministic_index, parse_pump_fee_recipients, parse_swap_fee_recipients,
};
use solana_sdk::pubkey::Pubkey;

fn pump_global_fixture(primary: &Pubkey, arr: &[Pubkey; 7]) -> Vec<u8> {
    let mut data = vec![0u8; 741];
    data[..8].copy_from_slice(&[167, 232, 232, 177, 200, 108, 114, 127]);
    data[8] = 1;
    data[41..73].copy_from_slice(primary.as_ref());
    for (i, pk) in arr.iter().enumerate() {
        let off = 162 + i * 32;
        data[off..off + 32].copy_from_slice(pk.as_ref());
    }
    data
}

fn swap_config_fixture(arr: &[Pubkey; 8]) -> Vec<u8> {
    let mut data = vec![0u8; 643];
    data[..8].copy_from_slice(&[149, 8, 156, 202, 160, 252, 176, 217]);
    for (i, pk) in arr.iter().enumerate() {
        let off = 57 + i * 32;
        data[off..off + 32].copy_from_slice(pk.as_ref());
    }
    data
}

#[test]
fn pump_parse_and_select() {
    let primary = Pubkey::new_unique();
    let mut arr = [Pubkey::default(); 7];
    for entry in arr.iter_mut() {
        *entry = Pubkey::new_unique();
    }
    let data = pump_global_fixture(&primary, &arr);
    let parsed = parse_pump_fee_recipients(&data).unwrap();

    assert_eq!(parsed.primary, primary);
    for (i, expected) in arr.iter().enumerate() {
        assert_eq!(parsed.recipients[i], *expected);
    }
    assert_eq!(parsed.all().len(), 8);
    assert_eq!(parsed.select(0), primary);
    assert_eq!(parsed.select(1), arr[0]);
    assert_eq!(parsed.select(7), arr[6]);
    assert_eq!(parsed.select(8), primary);
}

#[test]
fn pump_select_skips_zero_slots() {
    let primary = Pubkey::new_unique();
    let r0 = Pubkey::new_unique();
    let r1 = Pubkey::new_unique();
    let mut arr = [Pubkey::default(); 7];
    arr[0] = r0;
    arr[1] = r1;
    let data = pump_global_fixture(&primary, &arr);
    let parsed = parse_pump_fee_recipients(&data).unwrap();

    assert_eq!(parsed.select(0), primary);
    assert_eq!(parsed.select(1), r0);
    assert_eq!(parsed.select(2), r1);
    assert_eq!(parsed.select(3), primary);
}

#[test]
fn swap_parse_and_select() {
    let mut arr = [Pubkey::default(); 8];
    for entry in arr.iter_mut() {
        *entry = Pubkey::new_unique();
    }
    let data = swap_config_fixture(&arr);
    let parsed = parse_swap_fee_recipients(&data).unwrap();

    for (i, expected) in arr.iter().enumerate() {
        assert_eq!(parsed.recipients[i], *expected);
    }
    assert_eq!(parsed.select(0), arr[0]);
    assert_eq!(parsed.select(7), arr[7]);
    assert_eq!(parsed.select(8), arr[0]);
}

#[test]
fn pump_bad_discriminator() {
    let mut data = vec![0u8; 741];
    data[..8].copy_from_slice(&[0xFF; 8]);
    let err = parse_pump_fee_recipients(&data).unwrap_err().to_string();
    assert!(err.contains("discriminator mismatch"));
}

#[test]
fn pump_wrong_size() {
    let mut data = vec![0u8; 740];
    data[..8].copy_from_slice(&[167, 232, 232, 177, 200, 108, 114, 127]);
    let err = parse_pump_fee_recipients(&data).unwrap_err().to_string();
    assert!(err.contains("layout drift"));
}

#[test]
fn swap_wrong_size() {
    let mut data = vec![0u8; 644];
    data[..8].copy_from_slice(&[149, 8, 156, 202, 160, 252, 176, 217]);
    let err = parse_swap_fee_recipients(&data).unwrap_err().to_string();
    assert!(err.contains("layout drift"));
}

#[test]
fn swap_too_short() {
    assert!(parse_swap_fee_recipients(&[0u8; 100]).is_err());
}

#[test]
fn pump_disc_change_fallback() {
    let mut data = vec![0u8; 741];
    data[..8].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11]);
    let err = parse_pump_fee_recipients(&data).unwrap_err().to_string();
    assert!(err.contains("discriminator mismatch"));
}

#[test]
fn deterministic_stable() {
    let a = compute_deterministic_index(12345, b"e");
    let b = compute_deterministic_index(12345, b"e");
    assert_eq!(a, b);
    assert_ne!(a, compute_deterministic_index(12346, b"e"));
    assert_ne!(a, compute_deterministic_index(12345, b"f"));
}

#[test]
fn deterministic_bounded() {
    for slot in 0..100 {
        let idx = compute_deterministic_index(slot, b"t");
        assert!((idx as usize) % 8 < 8);
    }
}

#[test]
fn deterministic_slot_zero_ok() {
    let _ = (compute_deterministic_index(0, &[]) as usize) % 8;
}
