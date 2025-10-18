use borsa_alphavantage::AvConnector;
use borsa_core::AssetKind;
use borsa_core::BorsaConnector;

#[test]
fn av_connector_kind_matrix() {
    let av = AvConnector::new_with_key("DUMMY");

    // Alpha Vantage supports these asset kinds:
    assert!(av.supports_kind(AssetKind::Equity));
    assert!(av.supports_kind(AssetKind::Forex));
    assert!(av.supports_kind(AssetKind::Crypto));

    // Alpha Vantage does NOT support these asset kinds:
    assert!(!av.supports_kind(AssetKind::Fund));
    assert!(!av.supports_kind(AssetKind::Index));
    assert!(!av.supports_kind(AssetKind::Bond));
    assert!(!av.supports_kind(AssetKind::Commodity));
}
