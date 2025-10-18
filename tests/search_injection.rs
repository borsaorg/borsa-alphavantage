#![cfg(feature = "test-adapters")]

use std::sync::Arc;

use borsa_alphavantage::{AvConnector, adapter};
use borsa_core::connector::SearchProvider;
use borsa_core::{AssetKind, BorsaError, Exchange, SearchRequest, SearchResult, Symbol};

#[test]
fn search_injection_basic() {
    let _ = AvConnector::new_with_key("DUMMY");
    let ex = Exchange::try_from_str("US").unwrap();
    assert_eq!(ex.code(), "US");
}

struct A {
    s: Arc<dyn adapter::AvSearch>,
}
impl adapter::CloneArcAdapters for A {
    fn clone_arc_search(&self) -> Arc<dyn adapter::AvSearch> {
        self.s.clone()
    }
}

#[tokio::test]
async fn search_uses_injected_adapter() {
    let s = <dyn adapter::AvSearch>::from_fn(|_req| {
        Ok(vec![SearchResult {
            symbol: Symbol::new("AAPL").unwrap(),
            name: Some("Apple Inc".into()),
            exchange: Exchange::try_from_str("US").ok(),
            kind: AssetKind::Equity,
        }])
    });
    let av = AvConnector::from_adapter(&A { s });

    let req = SearchRequest::builder("apple").build().unwrap();
    let resp = av.search(req).await.unwrap();
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].symbol.as_str(), "AAPL");
}

#[tokio::test]
async fn search_not_found_is_mapped() {
    let s = <dyn adapter::AvSearch>::from_fn(|_r| {
        Err(BorsaError::connector(
            "borsa-alphavantage",
            "No matches found",
        ))
    });
    let av = AvConnector::from_adapter(&A { s });

    let req = SearchRequest::builder("zzz").build().unwrap();
    let err = av.search(req).await.err().unwrap();
    assert!(matches!(err, BorsaError::NotFound { .. }));
}

#[tokio::test]
async fn search_limit_is_respected() {
    let s = <dyn adapter::AvSearch>::from_fn(|_req| {
        let mut results = vec![];
        for i in 0..20 {
            results.push(SearchResult {
                symbol: Symbol::new(&format!("SYM{i}")).unwrap(),
                name: None,
                exchange: None,
                kind: AssetKind::Equity,
            });
        }
        Ok(results)
    });
    let av = AvConnector::from_adapter(&A { s });

    let req = SearchRequest::builder("x").limit(5).build().unwrap();
    let resp = av.search(req).await.unwrap();
    assert_eq!(resp.results.len(), 5);
}
