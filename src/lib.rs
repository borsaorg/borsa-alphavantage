//! Alpha Vantage connector for borsa.
//!
//! Provides quotes, history, search, and earnings via the `BorsaConnector` interface.
//!
//! Lightweight adapter helpers used in tests are behind the optional
//! `test-adapters` feature.
#![warn(missing_docs)]
use std::sync::Arc;

use async_trait::async_trait;

use borsa_core::{
    AssetKind, BorsaError, HistoryRequest, HistoryResponse, Instrument, Quote, SearchRequest,
    SearchResponse,
    connector::{
        BorsaConnector, ConnectorKey, /*EarningsProvider,*/ HistoryProvider, QuoteProvider,
        SearchProvider,
    },
};

/// Adapter layer that wraps the `alpha_vantage` client and exposes small async traits.
pub mod adapter;
mod convert;

#[cfg(feature = "test-adapters")]
use adapter::CloneArcAdapters;
use adapter::{/*AvEarnings,*/ AvHistory, AvQuotes, AvSearch, RealAdapter};

#[cfg(not(feature = "test-adapters"))]
type AdapterArc = Arc<RealAdapter>;

#[cfg(feature = "test-adapters")]
type QuotesAdapter = Arc<dyn AvQuotes>;
#[cfg(not(feature = "test-adapters"))]
type QuotesAdapter = AdapterArc;

#[cfg(feature = "test-adapters")]
type HistoryAdapter = Arc<dyn AvHistory>;
#[cfg(not(feature = "test-adapters"))]
type HistoryAdapter = AdapterArc;

#[cfg(feature = "test-adapters")]
type SearchAdapter = Arc<dyn AvSearch>;
#[cfg(not(feature = "test-adapters"))]
type SearchAdapter = AdapterArc;

/*
#[cfg(feature = "test-adapters")]
type EarningsAdapter = Arc<dyn AvEarnings>;
#[cfg(not(feature = "test-adapters"))]
type EarningsAdapter = AdapterArc;
*/

/// Public connector implementation backed by Alpha Vantage APIs.
pub struct AvConnector {
    quotes: QuotesAdapter,
    history: HistoryAdapter,
    search: SearchAdapter,
    /* earnings: EarningsAdapter, */
}

impl AvConnector {
    /// Use the native Alpha Vantage API key.
    pub fn new_with_key(key: impl Into<String>) -> Self {
        let a = RealAdapter::new_with_key(key);
        Self::from_adapter(&a)
    }

    /// Use a `RapidAPI` key for Alpha Vantage.
    pub fn new_with_rapidapi(key: impl Into<String>) -> Self {
        let a = RealAdapter::new_with_rapidapi(key);
        Self::from_adapter(&a)
    }

    /// Use the native Alpha Vantage API key with an external `reqwest::Client`.
    pub fn new_with_key_and_client(key: impl Into<String>, http: reqwest::Client) -> Self {
        let a = RealAdapter::new_with_key_and_client(key, http);
        Self::from_adapter(&a)
    }

    /// Use a `RapidAPI` key with an external `reqwest::Client`.
    pub fn new_with_rapidapi_and_client(key: impl Into<String>, http: reqwest::Client) -> Self {
        let a = RealAdapter::new_with_rapidapi_and_client(key, http);
        Self::from_adapter(&a)
    }

    /// For tests/injection.
    #[cfg(feature = "test-adapters")]
    #[must_use]
    pub fn from_adapter<A: CloneArcAdapters + 'static>(adapter: &A) -> Self {
        Self {
            quotes: adapter.clone_arc_quotes(),
            history: adapter.clone_arc_history(),
            search: adapter.clone_arc_search(),
            /* earnings: adapter.clone_arc_earnings(), */
        }
    }

    /// Build from a concrete `RealAdapter` by cloning it into shared handles.
    #[cfg(not(feature = "test-adapters"))]
    #[must_use]
    pub fn from_adapter(adapter: &RealAdapter) -> Self {
        let shared = Arc::new(adapter.clone());
        Self {
            quotes: Arc::clone(&shared),
            history: Arc::clone(&shared),
            search: Arc::clone(&shared),
            /* earnings: shared, */
        }
    }

    /// Static connector key used in orchestrator priority configuration.
    pub const KEY: ConnectorKey = ConnectorKey::new("borsa-alphavantage");

    fn looks_like_not_found(msg: &str) -> bool {
        let m = msg.to_ascii_lowercase();
        m.contains("invalid api call")
            || m.contains("no data")
            || m.contains("not found")
            || m.contains("unknown symbol")
            || m.contains("no matches")
    }

    fn normalize_error(e: BorsaError, what: &str) -> BorsaError {
        match e {
            BorsaError::Connector { connector, msg } => {
                if Self::looks_like_not_found(&msg) {
                    BorsaError::not_found(what.to_string())
                } else {
                    BorsaError::Connector { connector, msg }
                }
            }
            BorsaError::Other(msg) => {
                if Self::looks_like_not_found(&msg) {
                    BorsaError::not_found(what.to_string())
                } else {
                    BorsaError::connector("borsa-alphavantage", msg)
                }
            }
            other => other,
        }
    }

    /// Parse a forex symbol into base and quote currencies.
    /// Requires explicit delimiters: EUR/USD, BTC/USDT, etc.
    fn parse_forex_pair(symbol: &str) -> Result<(&str, &str), BorsaError> {
        // Try to split on common delimiters
        if let Some(pos) = symbol.find('/') {
            let base = &symbol[..pos];
            let quote = &symbol[pos + 1..];
            if base.is_empty() || quote.is_empty() {
                return Err(BorsaError::InvalidArg(format!(
                    "Invalid forex pair format: '{symbol}' - empty base or quote currency"
                )));
            }
            return Ok((base, quote));
        }

        if let Some(pos) = symbol.find('-') {
            let base = &symbol[..pos];
            let quote = &symbol[pos + 1..];
            if base.is_empty() || quote.is_empty() {
                return Err(BorsaError::InvalidArg(format!(
                    "Invalid forex pair format: '{symbol}' - empty base or quote currency"
                )));
            }
            return Ok((base, quote));
        }

        // No delimiter found - require explicit format
        Err(BorsaError::InvalidArg(format!(
            "Forex pair for AlphaVantage must be in 'BASE/QUOTE' format, got: '{symbol}'"
        )))
    }
}

#[async_trait]
impl QuoteProvider for AvConnector {
    async fn quote(&self, instrument: &Instrument) -> Result<Quote, BorsaError> {
        self.quotes
            .quote_equity(instrument.symbol_str())
            .await
            .map_err(|e| Self::normalize_error(e, &format!("quote for {}", instrument.symbol())))
    }
}

#[async_trait]
impl HistoryProvider for AvConnector {
    async fn history(
        &self,
        instrument: &Instrument,
        req: HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError> {
        match instrument.kind() {
            AssetKind::Forex => {
                let (base, quote) = Self::parse_forex_pair(instrument.symbol_str())?;
                self.history.forex(base, quote, &req).await.map_err(|e| {
                    Self::normalize_error(e, &format!("history for {}", instrument.symbol()))
                })
            }
            AssetKind::Crypto => self
                .history
                .crypto(instrument.symbol_str(), &req)
                .await
                .map_err(|e| {
                    Self::normalize_error(e, &format!("history for {}", instrument.symbol()))
                }),
            _ => self
                .history
                .equity(instrument.symbol_str(), &req)
                .await
                .map_err(|e| {
                    Self::normalize_error(e, &format!("history for {}", instrument.symbol()))
                }),
        }
    }

    fn supported_history_intervals(
        &self,
        _kind: AssetKind,
    ) -> &'static [borsa_core::types::Interval] {
        use borsa_core::types::Interval as I;
        const AV_INTERVALS: &[I] = &[
            I::I1m,
            I::I5m,
            I::I15m,
            I::I30m,
            I::I1h,
            I::D1,
            I::W1,
            I::M1,
        ];
        AV_INTERVALS
    }
}

#[async_trait]
impl SearchProvider for AvConnector {
    async fn search(&self, req: SearchRequest) -> Result<SearchResponse, BorsaError> {
        let mut results = self
            .search
            .search(&req)
            .await
            .map_err(|e| Self::normalize_error(e, "search"))?;
        if let Some(limit) = req.limit() {
            results.truncate(limit);
        }
        Ok(SearchResponse { results })
    }
}

/*
#[async_trait]
impl EarningsProvider for AvConnector {
    async fn earnings(&self, instrument: &Instrument) -> Result<borsa_core::Earnings, BorsaError> {
        self.earnings
            .earnings(instrument.symbol_str())
            .await
            .map_err(|e| Self::normalize_error(e, &format!("earnings for {}", instrument.symbol())))
    }
}
*/

#[async_trait]
impl BorsaConnector for AvConnector {
    fn name(&self) -> &'static str {
        "borsa-alphavantage"
    }

    fn vendor(&self) -> &'static str {
        "Alpha Vantage"
    }

    // capabilities removed; capability directory via as_*_provider

    fn as_history_provider(&self) -> Option<&dyn borsa_core::connector::HistoryProvider> {
        Some(self as &dyn HistoryProvider)
    }
    fn as_quote_provider(&self) -> Option<&dyn borsa_core::connector::QuoteProvider> {
        Some(self as &dyn QuoteProvider)
    }
    fn as_search_provider(&self) -> Option<&dyn borsa_core::connector::SearchProvider> {
        Some(self as &dyn SearchProvider)
    }
    // Earnings provider unfortunately returns an error from the underlying crate, doesnt seem to be a bug in the connector implementation.
    /*
    fn as_earnings_provider(&self) -> Option<&dyn borsa_core::connector::EarningsProvider> {
        Some(self as &dyn EarningsProvider)
    }
    */

    fn supports_kind(&self, kind: AssetKind) -> bool {
        matches!(
            kind,
            AssetKind::Equity | AssetKind::Forex | AssetKind::Crypto
        )
    }
}
