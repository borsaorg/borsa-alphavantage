#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{
    AssetKind, Candle, Currency, HistoryRequest, HistoryResponse, Instrument, Interval, Money,
    connector::HistoryProvider,
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
async fn crypto_symbol_handling() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| unreachable!(),
        |_b, _q, _r| unreachable!(),
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: chrono::Utc.timestamp_opt(100, 0).unwrap(),
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
    );
    let av = AvConnector::from_adapter(&A { h });

    let crypto_symbols = vec!["BTC", "ETH", "ADA", "DOT"];
    for symbol in crypto_symbols {
        let inst =
            Instrument::from_symbol(symbol, AssetKind::Crypto).expect("valid test instrument");
        let req = HistoryRequest::try_from_range(borsa_core::Range::D1, Interval::D1).unwrap();
        let response = av.history(&inst, req).await.unwrap();
        assert!(!response.candles.is_empty());
        assert!(!response.adjusted);
    }
}

#[tokio::test]
async fn crypto_different_intervals() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| unreachable!(),
        |_b, _q, _r| unreachable!(),
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: chrono::Utc.timestamp_opt(200, 0).unwrap(),
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
    );
    let av = AvConnector::from_adapter(&A2 { h });
    let inst = Instrument::from_symbol("BTC", AssetKind::Crypto).expect("valid test instrument");

    let intervals = vec![
        borsa_core::types::Interval::D1,
        borsa_core::types::Interval::W1,
        borsa_core::types::Interval::M1,
    ];
    for interval in intervals {
        let req = HistoryRequest::try_from_range(borsa_core::Range::D1, interval).unwrap();
        let response = av.history(&inst, req).await.unwrap();
        assert!(!response.candles.is_empty());
        assert!(!response.adjusted);
    }
}
