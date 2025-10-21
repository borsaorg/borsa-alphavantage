/*
#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{
    AssetKind, BorsaError, Currency, Earnings, EarningsQuarter, EarningsQuarterEps, EarningsYear,
    Instrument, Money, connector::EarningsProvider,
};

struct A {
    e: Arc<dyn adapter::AvEarnings>,
}
impl adapter::CloneArcAdapters for A {
    fn clone_arc_earnings(&self) -> Arc<dyn adapter::AvEarnings> {
        self.e.clone()
    }
}

#[tokio::test]
async fn earnings_uses_injected_adapter() {
    let e = <dyn adapter::AvEarnings>::from_fn(|_s| {
        Ok(Earnings {
            yearly: vec![EarningsYear {
                year: 2023,
                revenue: None,
                earnings: None,
            }],
            quarterly: vec![EarningsQuarter {
                period: "2023Q4".parse().unwrap(),
                revenue: None,
                earnings: None,
            }],
            quarterly_eps: vec![EarningsQuarterEps {
                period: "2023Q4".parse().unwrap(),
                actual: Some(
                    Money::from_canonical_str("2.0", Currency::Iso(borsa_core::IsoCurrency::USD))
                        .unwrap(),
                ),
                estimate: Some(
                    Money::from_canonical_str("1.9", Currency::Iso(borsa_core::IsoCurrency::USD))
                        .unwrap(),
                ),
            }],
        })
    });
    let av = AvConnector::from_adapter(&A { e });
    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let earnings = av.earnings(&inst).await.unwrap();
    assert_eq!(earnings.yearly.len(), 1);
}


#[tokio::test]
async fn earnings_not_found_is_mapped() {
    let e = <dyn adapter::AvEarnings>::from_fn(|_s| {
        Err(BorsaError::connector(
            "borsa-alphavantage",
            "No data for symbol",
        ))
    });
    let av = AvConnector::from_adapter(&A { e });
    let inst =
        Instrument::from_symbol("MISSING", AssetKind::Equity).expect("valid test instrument");
    let err = av.earnings(&inst).await.err().unwrap();
    assert!(matches!(err, BorsaError::NotFound { .. }));
}
*/