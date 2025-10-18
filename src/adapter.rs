use std::sync::Arc;

use async_trait::async_trait;
use borsa_core::{
    BorsaError, Earnings, EarningsQuarter, EarningsQuarterEps, EarningsYear, HistoryRequest,
    HistoryResponse, Quote, SearchRequest, SearchResult, Symbol,
};
use borsa_core::{Currency, Money};

use crate::convert::{
    map_crypto_to_history, map_forex_to_history, map_kind_from_search_type,
    map_timeseries_to_history,
};
use alpha_vantage as av;

/// Quotes adapter abstraction wrapping Alpha Vantage quote endpoint(s).
#[async_trait]
pub trait AvQuotes: Send + Sync {
    /// Fetch a single equity quote by symbol.
    async fn quote_equity(&self, symbol: &str) -> Result<Quote, BorsaError>;
}

/// History adapter abstraction wrapping Alpha Vantage time-series endpoints.
#[async_trait]
pub trait AvHistory: Send + Sync {
    /// Fetch equity OHLCV history.
    async fn equity(
        &self,
        symbol: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError>;
    /// Fetch forex OHLCV history given base and quote currency.
    async fn forex(
        &self,
        base: &str,
        quote: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError>;
    /// Fetch crypto OHLCV history.
    async fn crypto(
        &self,
        symbol: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError>;
}

/// Search adapter abstraction wrapping Alpha Vantage symbol search.
#[async_trait]
pub trait AvSearch: Send + Sync {
    /// Perform a symbol search and return raw Alpha Vantage matches.
    async fn search(&self, req: &SearchRequest) -> Result<Vec<SearchResult>, BorsaError>;
}

/// Fundamentals adapter for fetching earnings data.
#[async_trait]
pub trait AvEarnings: Send + Sync {
    /// Fetch earnings for the provided symbol.
    async fn earnings(&self, symbol: &str) -> Result<Earnings, BorsaError>;
}

/// Production adapter that owns an `alpha_vantage::ApiClient`.
#[derive(Clone)]
pub struct RealAdapter {
    client: Arc<av::ApiClient>,
}

impl RealAdapter {
    fn make_default_client() -> reqwest::Client {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("failed to build reqwest client without system proxy")
    }

    /// Build using the native Alpha Vantage API key and an internal client.
    pub fn new_with_key(key: impl Into<String>) -> Self {
        let client = Self::make_default_client();
        let api = av::set_api(key, client);
        Self {
            client: Arc::new(api),
        }
    }
    /// Build using a `RapidAPI` key for Alpha Vantage and an internal client.
    pub fn new_with_rapidapi(key: impl Into<String>) -> Self {
        let client = Self::make_default_client();
        let api = av::set_rapid_api(key, client);
        Self {
            client: Arc::new(api),
        }
    }

    /// Build using an external `reqwest::Client` with the native Alpha Vantage API key.
    pub fn new_with_key_and_client(key: impl Into<String>, client: reqwest::Client) -> Self {
        let api = av::set_api(key, client);
        Self {
            client: Arc::new(api),
        }
    }

    /// Build using an external `reqwest::Client` with a `RapidAPI` key for Alpha Vantage.
    pub fn new_with_rapidapi_and_client(key: impl Into<String>, client: reqwest::Client) -> Self {
        let api = av::set_rapid_api(key, client);
        Self {
            client: Arc::new(api),
        }
    }
}

#[async_trait]
impl AvQuotes for RealAdapter {
    async fn quote_equity(&self, symbol: &str) -> Result<Quote, BorsaError> {
        let q = self
            .client
            .quote(symbol)
            .json()
            .await
            .map_err(|e| BorsaError::connector("borsa-alphavantage", e.to_string()))?;

        let sym = q.symbol().to_string();
        if sym.is_empty() {
            return Err(BorsaError::not_found(format!("quote for {symbol}")));
        }
        let symbol = Symbol::new(&sym)
            .map_err(|e| BorsaError::Data(format!("invalid symbol '{sym}': {e}")))?;
        let to_money = |v: f64| {
            Money::from_canonical_str(&v.to_string(), Currency::Iso(borsa_core::IsoCurrency::USD))
                .unwrap()
        };
        Ok(Quote {
            symbol,
            shortname: None,
            price: Some(to_money(q.price())),
            previous_close: Some(to_money(q.previous())),
            exchange: None,
            market_state: None,
        })
    }
}

#[async_trait]
impl AvHistory for RealAdapter {
    async fn equity(
        &self,
        symbol: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError> {
        use av::api::{OutputSize, TimeSeriesInterval as A};
        use av::stock_time::StockFunction as SF;

        let (func, interval) = match req.interval() {
            i if i.is_intraday() => {
                let iv = match i.minutes() {
                    Some(1) => A::OneMin,
                    Some(5) => A::FiveMin,
                    Some(15) => A::FifteenMin,
                    Some(30) => A::ThirtyMin,
                    Some(60) => A::SixtyMin,
                    _ => {
                        return Err(BorsaError::unsupported(
                            "intraday interval for Alpha Vantage",
                        ));
                    }
                };
                (SF::IntraDay, Some(iv))
            }
            borsa_core::Interval::D1 => {
                if req.auto_adjust() {
                    (SF::DailyAdjusted, None)
                } else {
                    (SF::Daily, None)
                }
            }
            borsa_core::Interval::W1 => {
                if req.auto_adjust() {
                    (SF::WeeklyAdjusted, None)
                } else {
                    (SF::Weekly, None)
                }
            }
            borsa_core::Interval::M1 => {
                if req.auto_adjust() {
                    (SF::MonthlyAdjusted, None)
                } else {
                    (SF::Monthly, None)
                }
            }
            _ => return Err(BorsaError::unsupported("interval for Alpha Vantage")),
        };

        let mut b = self.client.stock_time(func, symbol);
        if let Some(iv) = interval {
            b = b.interval(iv);
            b = b.adjusted(req.auto_adjust());
        }
        b = b.output_size(OutputSize::Full);

        let ts = b
            .json()
            .await
            .map_err(|e| BorsaError::connector("borsa-alphavantage", e.to_string()))?;
        Ok(map_timeseries_to_history(&ts))
    }

    async fn forex(
        &self,
        base: &str,
        quote: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError> {
        use av::api::{OutputSize, TimeSeriesInterval as A};
        use av::forex::ForexFunction as FF;

        let (func, intr) = match req.interval() {
            i if i.is_intraday() => {
                let iv = match i.minutes() {
                    Some(1) => A::OneMin,
                    Some(5) => A::FiveMin,
                    Some(15) => A::FifteenMin,
                    Some(30) => A::ThirtyMin,
                    Some(60) => A::SixtyMin,
                    _ => {
                        return Err(BorsaError::unsupported(
                            "intraday interval for Alpha Vantage",
                        ));
                    }
                };
                (FF::IntraDay, Some(iv))
            }
            borsa_core::Interval::D1 => (FF::Daily, None),
            borsa_core::Interval::W1 => (FF::Weekly, None),
            borsa_core::Interval::M1 => (FF::Monthly, None),
            _ => return Err(BorsaError::unsupported("interval for Alpha Vantage")),
        };

        let mut b = self.client.forex(func, base, quote);
        if let Some(i) = intr {
            b = b.interval(i);
        }
        b = b.output_size(OutputSize::Full);

        let fx = b
            .json()
            .await
            .map_err(|e| BorsaError::connector("borsa-alphavantage", e.to_string()))?;
        Ok(map_forex_to_history(&fx))
    }

    async fn crypto(
        &self,
        symbol: &str,
        req: &HistoryRequest,
    ) -> Result<HistoryResponse, BorsaError> {
        use av::crypto::CryptoFunction as CF;
        let func = match req.interval() {
            borsa_core::Interval::W1 => CF::Weekly,
            borsa_core::Interval::M1 | borsa_core::Interval::M3 => CF::Monthly,
            _ => CF::Daily,
        };
        let c = self
            .client
            .crypto(func, symbol, "USD")
            .json()
            .await
            .map_err(|e| BorsaError::connector("borsa-alphavantage", e.to_string()))?;
        Ok(map_crypto_to_history(&c))
    }
}

#[async_trait]
impl AvSearch for RealAdapter {
    async fn search(&self, req: &SearchRequest) -> Result<Vec<SearchResult>, BorsaError> {
        let result = self
            .client
            .search(req.query())
            .json()
            .await
            .map_err(|e| BorsaError::connector("borsa-alphavantage", e.to_string()))?;

        let mut out: Vec<SearchResult> = Vec::new();
        for m in result.matches() {
            let kind = map_kind_from_search_type(m.stock_type());
            if let Some(k) = req.kind()
                && k != kind
            {
                continue;
            }
            out.push(SearchResult {
                symbol: Symbol::new(m.symbol()).map_err(|e| {
                    BorsaError::Data(format!("invalid symbol '{}': {e}", m.symbol()))
                })?,
                name: Some(m.name().to_string()),
                exchange: borsa_core::Exchange::try_from_str(m.region()).ok(),
                kind,
            });
        }

        if let Some(limit) = req.limit()
            && out.len() > limit
        {
            out.truncate(limit);
        }

        Ok(out)
    }
}

#[async_trait]
impl AvEarnings for RealAdapter {
    async fn earnings(&self, symbol: &str) -> Result<Earnings, BorsaError> {
        let e = self
            .client
            .earning(symbol)
            .json()
            .await
            .map_err(|er| BorsaError::connector("borsa-alphavantage", er.to_string()))?;

        let mut yearly: Vec<EarningsYear> = Vec::new();
        for y in e.annual_earning() {
            let year = y
                .fiscal_date_ending()
                .get(..4)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            yearly.push(EarningsYear {
                year,
                revenue: None,
                earnings: None,
            });
        }

        let mut quarterly_eps: Vec<EarningsQuarterEps> = Vec::new();
        let mut quarterly: Vec<EarningsQuarter> = Vec::new();
        for q in e.quarterly_earning() {
            use borsa_core::{Currency, Money};
            let to_money = |v: f64| {
                Money::from_canonical_str(
                    &v.to_string(),
                    Currency::Iso(borsa_core::IsoCurrency::USD),
                )
                .unwrap()
            };
            quarterly_eps.push(EarningsQuarterEps {
                period: if q.reported_date().is_empty() {
                    q.fiscal_date_ending()
                        .parse::<borsa_core::Period>()
                        .unwrap_or(borsa_core::Period::Year { year: 0 })
                } else {
                    q.reported_date()
                        .parse::<borsa_core::Period>()
                        .unwrap_or(borsa_core::Period::Year { year: 0 })
                },
                actual: q.reported_eps().map(to_money),
                estimate: Some(to_money(q.estimated_eps())),
            });
            quarterly.push(EarningsQuarter {
                period: q
                    .fiscal_date_ending()
                    .parse::<borsa_core::Period>()
                    .unwrap_or(borsa_core::Period::Year { year: 0 }),
                revenue: None,
                earnings: None,
            });
        }

        Ok(Earnings {
            yearly,
            quarterly,
            quarterly_eps,
        })
    }
}

/* -------- Test-only lightweight adapter constructors ------- */

#[cfg(feature = "test-adapters")]
impl dyn AvQuotes {
    /// Build an `AvQuotes` from a closure (tests only).
    pub fn from_fn<F>(f: F) -> Arc<dyn AvQuotes>
    where
        F: Send + Sync + 'static + Fn(String) -> Result<Quote, BorsaError>,
    {
        struct FnQuotes<F>(F);
        #[async_trait]
        impl<F> AvQuotes for FnQuotes<F>
        where
            F: Send + Sync + 'static + Fn(String) -> Result<Quote, BorsaError>,
        {
            async fn quote_equity(&self, symbol: &str) -> Result<Quote, BorsaError> {
                (self.0)(symbol.to_string())
            }
        }
        Arc::new(FnQuotes(f))
    }
}

#[cfg(feature = "test-adapters")]
impl dyn AvHistory {
    /// Build an `AvHistory` from closures (tests only).
    pub fn from_fns<FE, FFx, FC>(fe: FE, ffx: FFx, fc: FC) -> Arc<dyn AvHistory>
    where
        FE: Send
            + Sync
            + 'static
            + Fn(String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
        FFx: Send
            + Sync
            + 'static
            + Fn(String, String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
        FC: Send
            + Sync
            + 'static
            + Fn(String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
    {
        struct FnHistory<FE, FFx, FC> {
            fe: FE,
            ffx: FFx,
            fc: FC,
        }
        #[async_trait]
        impl<FE, FFx, FC> AvHistory for FnHistory<FE, FFx, FC>
        where
            FE: Send
                + Sync
                + 'static
                + Fn(String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
            FFx: Send
                + Sync
                + 'static
                + Fn(String, String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
            FC: Send
                + Sync
                + 'static
                + Fn(String, HistoryRequest) -> Result<HistoryResponse, BorsaError>,
        {
            async fn equity(
                &self,
                symbol: &str,
                req: &HistoryRequest,
            ) -> Result<HistoryResponse, BorsaError> {
                (self.fe)(symbol.to_string(), req.clone())
            }
            async fn forex(
                &self,
                base: &str,
                quote: &str,
                req: &HistoryRequest,
            ) -> Result<HistoryResponse, BorsaError> {
                (self.ffx)(base.to_string(), quote.to_string(), req.clone())
            }
            async fn crypto(
                &self,
                symbol: &str,
                req: &HistoryRequest,
            ) -> Result<HistoryResponse, BorsaError> {
                (self.fc)(symbol.to_string(), req.clone())
            }
        }
        Arc::new(FnHistory { fe, ffx, fc })
    }
}

#[cfg(feature = "test-adapters")]
impl dyn AvSearch {
    /// Build an `AvSearch` from a closure (tests only).
    pub fn from_fn<F>(f: F) -> Arc<dyn AvSearch>
    where
        F: Send + Sync + 'static + Fn(SearchRequest) -> Result<Vec<SearchResult>, BorsaError>,
    {
        struct FnSearch<F>(F);
        #[async_trait]
        impl<F> AvSearch for FnSearch<F>
        where
            F: Send + Sync + 'static + Fn(SearchRequest) -> Result<Vec<SearchResult>, BorsaError>,
        {
            async fn search(&self, req: &SearchRequest) -> Result<Vec<SearchResult>, BorsaError> {
                (self.0)(req.clone())
            }
        }
        Arc::new(FnSearch(f))
    }
}

#[cfg(feature = "test-adapters")]
impl dyn AvEarnings {
    /// Build an `AvEarnings` from a closure (tests only).
    pub fn from_fn<F>(f: F) -> Arc<dyn AvEarnings>
    where
        F: Send + Sync + 'static + Fn(String) -> Result<Earnings, BorsaError>,
    {
        struct FnEarnings<F>(F);
        #[async_trait]
        impl<F> AvEarnings for FnEarnings<F>
        where
            F: Send + Sync + 'static + Fn(String) -> Result<Earnings, BorsaError>,
        {
            async fn earnings(&self, symbol: &str) -> Result<Earnings, BorsaError> {
                (self.0)(symbol.to_string())
            }
        }
        Arc::new(FnEarnings(f))
    }
}

/// Helper trait to split a concrete adapter into arc trait objects.
#[cfg(feature = "test-adapters")]
pub trait CloneArcAdapters {
    /// Clone as `Arc<dyn AvQuotes>`.
    fn clone_arc_quotes(&self) -> Arc<dyn AvQuotes> {
        <dyn AvQuotes>::from_fn(|_s| Err(BorsaError::unsupported("quote")))
    }
    /// Clone as `Arc<dyn AvHistory>`.
    fn clone_arc_history(&self) -> Arc<dyn AvHistory> {
        <dyn AvHistory>::from_fns(
            |_s, _r| Err(BorsaError::unsupported("history/equity")),
            |_b, _q, _r| Err(BorsaError::unsupported("history/forex")),
            |_s, _r| Err(BorsaError::unsupported("history/crypto")),
        )
    }
    /// Clone as `Arc<dyn AvSearch>`.
    fn clone_arc_search(&self) -> Arc<dyn AvSearch> {
        <dyn AvSearch>::from_fn(|_r| Err(BorsaError::unsupported("search")))
    }
    /// Clone as `Arc<dyn AvEarnings>`.
    fn clone_arc_earnings(&self) -> Arc<dyn AvEarnings> {
        <dyn AvEarnings>::from_fn(|_s| Err(BorsaError::unsupported("fundamentals/earnings")))
    }
}

#[cfg(feature = "test-adapters")]
impl CloneArcAdapters for RealAdapter {
    /// Clone as `Arc<dyn AvQuotes>`.
    fn clone_arc_quotes(&self) -> Arc<dyn AvQuotes> {
        Arc::new(self.clone()) as Arc<dyn AvQuotes>
    }
    /// Clone as `Arc<dyn AvHistory>`.
    fn clone_arc_history(&self) -> Arc<dyn AvHistory> {
        Arc::new(self.clone()) as Arc<dyn AvHistory>
    }
    /// Clone as `Arc<dyn AvSearch>`.
    fn clone_arc_search(&self) -> Arc<dyn AvSearch> {
        Arc::new(self.clone()) as Arc<dyn AvSearch>
    }
    /// Clone as `Arc<dyn AvEarnings>`.
    fn clone_arc_earnings(&self) -> Arc<dyn AvEarnings> {
        Arc::new(self.clone()) as Arc<dyn AvEarnings>
    }
}
