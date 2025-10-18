#![cfg(feature = "test-adapters")]

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{AssetKind, BorsaConnector, types::Interval};

#[test]
fn av_supported_history_intervals_are_sane() {
    // No network call happens by just constructing the connector.
    struct A;
    impl adapter::CloneArcAdapters for A {}
    let av = AvConnector::from_adapter(&A);

    let list = av
        .as_history_provider()
        .unwrap()
        .supported_history_intervals(AssetKind::Equity);

    // Intraday that AV actually supports:
    assert!(list.contains(&Interval::I1m));
    assert!(list.contains(&Interval::I5m));
    assert!(list.contains(&Interval::I15m));
    assert!(list.contains(&Interval::I30m));
    assert!(list.contains(&Interval::I1h));

    // Higher-level granularities:
    assert!(list.contains(&Interval::D1));
    assert!(list.contains(&Interval::W1));
    assert!(list.contains(&Interval::M1));

    // Intervals AV does NOT advertise:
    assert!(!list.contains(&Interval::I2m));
    assert!(!list.contains(&Interval::I90m));
    assert!(!list.contains(&Interval::D5));
    assert!(!list.contains(&Interval::M3));
}

#[test]
fn av_supports_expected_kinds() {
    struct A;
    impl adapter::CloneArcAdapters for A {}
    let av = AvConnector::from_adapter(&A);

    assert!(av.supports_kind(AssetKind::Equity));
    assert!(av.supports_kind(AssetKind::Forex));
    assert!(av.supports_kind(AssetKind::Crypto));

    assert!(!av.supports_kind(AssetKind::Fund));
    assert!(!av.supports_kind(AssetKind::Index));
    assert!(!av.supports_kind(AssetKind::Bond));
    assert!(!av.supports_kind(AssetKind::Commodity));
}
