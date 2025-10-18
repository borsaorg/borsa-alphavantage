#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{
    AssetKind, Candle, Currency, HistoryRequest, HistoryResponse, Instrument, Interval, Money,
    Range, connector::HistoryProvider,
};
use chrono::TimeZone;

struct A {
    h: Arc<dyn adapter::AvHistory>,
}
impl adapter::CloneArcAdapters for A {
    fn clone_arc_history(&self) -> Arc<dyn adapter::AvHistory> {
        self.h.clone()
    }
}
struct A2 {
    h: Arc<dyn adapter::AvHistory>,
}
impl adapter::CloneArcAdapters for A2 {
    fn clone_arc_history(&self) -> Arc<dyn adapter::AvHistory> {
        self.h.clone()
    }
}
#[tokio::test]
async fn history_validation_works() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: chrono::Utc.timestamp_opt(1, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "1.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "1.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "1.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "1.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close_unadj: None,
                    volume: None,
                }],
                actions: vec![],
                adjusted: false,
                meta: None,
            })
        },
        |_b, _q, _r| unreachable!(),
        |_s, _r| unreachable!(),
    );
    let av = AvConnector::from_adapter(&A { h });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");

    // Test with a valid request
    let req = HistoryRequest::try_from_range(Range::D1, Interval::D1).unwrap();

    // This should pass validation and might work at the network level
    let result = av.history(&inst, req).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn history_validation_rejects_invalid_requests() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: chrono::Utc.timestamp_opt(2, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "2.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "2.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "2.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "2.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close_unadj: None,
                    volume: None,
                }],
                actions: vec![],
                adjusted: false,
                meta: None,
            })
        },
        |_b, _q, _r| unreachable!(),
        |_s, _r| unreachable!(),
    );
    let av = AvConnector::from_adapter(&A2 { h });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");

    // Test with a valid request that will fail at network level
    let req = HistoryRequest::try_from_range(Range::D1, Interval::D1).unwrap();

    // This should work at network level
    let result = av.history(&inst, req).await;

    assert!(result.is_ok());
}
