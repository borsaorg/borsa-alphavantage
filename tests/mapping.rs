use borsa_alphavantage::AvConnector;
use borsa_core::{AssetKind, BorsaConnector};

#[test]
fn mapping_smoke() {
    let av = AvConnector::from_adapter(&borsa_alphavantage::adapter::RealAdapter::new_with_key(
        "test",
    ));
    // Just ensure construction works without network
    let _ = av.name();
}

#[test]
fn mapping_smoke_again() {
    let av = AvConnector::from_adapter(&borsa_alphavantage::adapter::RealAdapter::new_with_key(
        "test",
    ));
    let _ = av.vendor();
}

#[test]
fn mapping_supports_known_kinds() {
    let av = AvConnector::from_adapter(&borsa_alphavantage::adapter::RealAdapter::new_with_key(
        "test",
    ));
    assert!(av.supports_kind(AssetKind::Equity));
}
