#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{
    AssetKind, Candle, HistoryRequest, HistoryResponse, Instrument, connector::HistoryProvider,
};
use borsa_core::{Currency, Interval, Money};
use chrono::{TimeZone, Utc};

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
struct A3 {
    h: Arc<dyn adapter::AvHistory>,
}
impl adapter::CloneArcAdapters for A3 {
    fn clone_arc_history(&self) -> Arc<dyn adapter::AvHistory> {
        self.h.clone()
    }
}
#[tokio::test]
async fn intraday_intervals_supported() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: Utc.timestamp_opt(1, 0).unwrap(),
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

    let supported_intervals = vec![
        Interval::I1m,
        Interval::I5m,
        Interval::I15m,
        Interval::I30m,
        Interval::I1h,
    ];

    for interval in supported_intervals {
        let req = HistoryRequest::try_from_range(borsa_core::Range::M1, interval).unwrap();
        let response = av.history(&inst, req).await.unwrap();
        assert!(!response.candles.is_empty());
    }
}

#[tokio::test]
async fn daily_intervals_supported() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: Utc.timestamp_opt(10, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "10.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "10.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "10.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "10.0",
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

    let req_adjusted = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::D1).unwrap();
    let req_raw = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::D1).unwrap();
    assert!(av.history(&inst, req_adjusted).await.is_ok());
    assert!(av.history(&inst, req_raw).await.is_ok());
}

#[tokio::test]
async fn weekly_intervals_supported() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: Utc.timestamp_opt(20, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "20.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "20.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "20.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "20.0",
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
    let req_adjusted = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::W1).unwrap();
    let req_raw = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::W1).unwrap();
    assert!(av.history(&inst, req_adjusted).await.is_ok());
    assert!(av.history(&inst, req_raw).await.is_ok());
}

#[tokio::test]
async fn monthly_intervals_supported() {
    let h = <dyn adapter::AvHistory>::from_fns(
        |_s, _r| {
            Ok(HistoryResponse {
                candles: vec![Candle {
                    ts: Utc.timestamp_opt(30, 0).unwrap(),
                    open: Money::from_canonical_str(
                        "30.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    high: Money::from_canonical_str(
                        "30.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    low: Money::from_canonical_str(
                        "30.0",
                        Currency::Iso(borsa_core::IsoCurrency::USD),
                    )
                    .unwrap(),
                    close: Money::from_canonical_str(
                        "30.0",
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
    let av = AvConnector::from_adapter(&A3 { h });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let req_adjusted = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::M1).unwrap();
    let req_raw = HistoryRequest::try_from_range(borsa_core::Range::M1, Interval::M1).unwrap();
    assert!(av.history(&inst, req_adjusted).await.is_ok());
    assert!(av.history(&inst, req_raw).await.is_ok());
}
