#![cfg(feature = "test-adapters")]

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::BorsaConnector;

#[test]
fn av_connector_advertises_expected_capabilities() {
    // Build with minimal injected adapters
    struct A;
    impl adapter::CloneArcAdapters for A {}
    let av = AvConnector::from_adapter(&A);
    // With capability directory, presence is checked via as_*_provider
    assert!(av.as_quote_provider().is_some());
    assert!(av.as_history_provider().is_some());
    assert!(av.as_search_provider().is_some());
    assert!(av.as_earnings_provider().is_some());
    assert!(av.as_profile_provider().is_none());
    assert!(av.as_options_expirations_provider().is_none());
    assert!(av.as_option_chain_provider().is_none());
    assert!(av.as_recommendations_provider().is_none());
    assert!(av.as_recommendations_summary_provider().is_none());
    assert!(av.as_upgrades_downgrades_provider().is_none());
    assert!(av.as_analyst_price_target_provider().is_none());
    assert!(av.as_major_holders_provider().is_none());
    assert!(av.as_institutional_holders_provider().is_none());
    assert!(av.as_mutual_fund_holders_provider().is_none());
    assert!(av.as_insider_transactions_provider().is_none());
    assert!(av.as_insider_roster_holders_provider().is_none());
    assert!(av.as_net_share_purchase_activity_provider().is_none());
    assert!(av.as_esg_provider().is_none());
    assert!(av.as_news_provider().is_none());
}
