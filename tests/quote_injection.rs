#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::{AssetKind, BorsaConnector, BorsaError, Instrument, connector::QuoteProvider};

struct TestAdapter {
    q: Arc<dyn adapter::AvQuotes>,
}

impl adapter::CloneArcAdapters for TestAdapter {
    fn clone_arc_quotes(&self) -> Arc<dyn adapter::AvQuotes> {
        self.q.clone()
    }
}

#[tokio::test]
async fn quote_equity_maps_correctly() {
    let q = <dyn adapter::AvQuotes>::from_fn(|s| {
        use borsa_core::{Currency, Money};
        let to_money = |v: f64| {
            Money::from_canonical_str(&v.to_string(), Currency::Iso(borsa_core::IsoCurrency::USD))
                .unwrap()
        };
        Ok(borsa_core::Quote {
            symbol: borsa_core::Symbol::new(&s).unwrap(),
            shortname: None,
            price: Some(to_money(123.0)),
            previous_close: Some(to_money(120.0)),
            exchange: Some(borsa_core::Exchange::NASDAQ),
            market_state: None,
        })
    });

    let av = AvConnector::from_adapter(&TestAdapter { q });

    assert_eq!(av.name(), "borsa-alphavantage");
    assert!(av.as_quote_provider().is_some());

    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let quote = av.quote(&inst).await.unwrap();
    assert_eq!(quote.symbol.as_str(), "AAPL");
    assert!(quote.price.is_some());
}

#[tokio::test]
async fn quote_not_found_is_mapped() {
    let q = <dyn adapter::AvQuotes>::from_fn(|_s| {
        Err(BorsaError::connector(
            "borsa-alphavantage",
            "Invalid API call. Unknown symbol",
        ))
    });
    let av = AvConnector::from_adapter(&TestAdapter { q });

    let inst =
        Instrument::from_symbol("ZZZZ-UNKNOWN", AssetKind::Equity).expect("valid test instrument");
    let err = av.quote(&inst).await.err().unwrap();
    assert!(matches!(err, BorsaError::NotFound { .. }));
}

#[tokio::test]
async fn quote_error_preserves_connector_name() {
    let q = <dyn adapter::AvQuotes>::from_fn(|_s| {
        Err(BorsaError::Other(
            "We have detected your API key as *** and our standard API rate limit is 25 requests per day. Please subscribe to any of the premium plans at https://www.alphavantage.co/premium/ to instantly remove all daily rate limits.".to_string()
        ))
    });
    let av = AvConnector::from_adapter(&TestAdapter { q });

    let inst = Instrument::from_symbol("AAPL", AssetKind::Equity).expect("valid test instrument");
    let err = av.quote(&inst).await.err().unwrap();

    // The error should be converted to a Connector error with the correct connector name
    match err {
        BorsaError::Connector { connector, msg } => {
            assert_eq!(connector, "borsa-alphavantage");
            assert!(msg.contains("API rate limit"));
        }
        _ => panic!("Expected Connector error, got: {err:?}"),
    }
}
