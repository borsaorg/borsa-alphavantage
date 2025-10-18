#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{
    AssetKind, BorsaError, Candle, Currency, HistoryRequest, HistoryResponse, Instrument, Interval,
    Money, Range, connector::HistoryProvider,
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
#[tokio::test]
async fn history_equity_maps_correctly() {
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
                adjusted: true,
                meta: None,
            })
        },
        |_b, _q, _r| unreachable!(),
        |_s, _r| unreachable!(),
    );
    let av = AvConnector::from_adapter(&A { h });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let req = HistoryRequest::try_from_range(Range::D1, Interval::D1).unwrap();
    let response = av.history(&inst, req).await.unwrap();
    assert!(!response.candles.is_empty());
}

#[tokio::test]
async fn history_forex_maps_correctly() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| unreachable!(),
        |_b, _q, _r| {
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
        |_s, _r| unreachable!(),
    );
    let av = AvConnector::from_adapter(&A { h });
    let inst = Instrument::from_symbol("EUR/USD", AssetKind::Forex).expect("valid test instrument");
    let req = HistoryRequest::try_from_range(Range::D1, Interval::D1).unwrap();
    let response = av.history(&inst, req).await.unwrap();
    assert!(!response.candles.is_empty());
}

#[tokio::test]
async fn history_crypto_maps_correctly() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| unreachable!(),
        |_b, _q, _r| unreachable!(),
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: chrono::Utc.timestamp_opt(3, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "3.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "3.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "3.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "3.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close_unadj: None,
                    volume: Some(1),
                }],
                actions: vec![],
                adjusted: false,
                meta: None,
            })
        },
    );
    let av = AvConnector::from_adapter(&A { h });
    let inst = Instrument::from_symbol("BTC", AssetKind::Crypto).expect("valid test instrument");
    let req = HistoryRequest::try_from_range(Range::D1, Interval::D1).unwrap();
    let response = av.history(&inst, req).await.unwrap();
    assert!(!response.candles.is_empty());
}

#[tokio::test]
async fn history_unsupported_interval_returns_error() {
    // Adapter returns Unsupported for equity 2m
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, r| {
            if r.interval() == Interval::I2m {
                Err(BorsaError::unsupported("interval"))
            } else {
                unreachable!()
            }
        },
        |_b, _q, _r| unreachable!(),
        |_s, _r| unreachable!(),
    );
    let av = AvConnector::from_adapter(&A { h });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let req = HistoryRequest::try_from_range(Range::D1, Interval::I2m).unwrap();
    let err = av.history(&inst, req).await.err().unwrap();
    assert!(format!("{err}").to_lowercase().contains("unsupported"));
}
