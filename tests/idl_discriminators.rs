use pump_cli::core::instructions;

#[test]
fn buy() {
    assert_eq!(
        instructions::discriminator("buy"),
        instructions::PUMP_BUY_DISCRIMINATOR
    );
}

#[test]
fn sell() {
    assert_eq!(
        instructions::discriminator("sell"),
        instructions::PUMP_SELL_DISCRIMINATOR
    );
}

#[test]
fn create() {
    assert_eq!(
        instructions::discriminator("create"),
        instructions::PUMP_CREATE_DISCRIMINATOR
    );
}

#[test]
fn create_v2() {
    assert_eq!(
        instructions::discriminator("create_v2"),
        instructions::PUMP_CREATE_V2_DISCRIMINATOR
    );
}

#[test]
fn bonding_curve_valid() {
    use pump_cli::core::bonding_curve::BondingCurve;
    let mut data = vec![0u8; 8 + 75];
    data[..8].copy_from_slice(&[23, 183, 248, 55, 96, 216, 172, 96]);
    assert!(BondingCurve::validate_discriminator(&data).is_ok());
    data[0] = 0xFF;
    assert!(BondingCurve::validate_discriminator(&data).is_err());
}

#[test]
fn swap_pool_valid() {
    use pump_cli::core::pump_swap::SwapPool;
    let mut data = vec![0u8; 8 + 237];
    data[..8].copy_from_slice(&[241, 154, 109, 4, 17, 177, 109, 188]);
    assert!(SwapPool::validate_discriminator(&data).is_ok());
    data[0] = 0xFF;
    assert!(SwapPool::validate_discriminator(&data).is_err());
}
