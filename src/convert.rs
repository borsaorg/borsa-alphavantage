use std::str::FromStr;

use alpha_vantage as av;
use borsa_core::{self, AssetKind, Candle, Currency, HistoryMeta, HistoryResponse, Money};
use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

fn round_non_negative_f64_to_u64_saturating(v: f64) -> u64 {
    if !v.is_finite() {
        return u64::MAX;
    }
    if v <= 0.0 {
        return 0;
    }
    let r = v.round();
    let s = format!("{r:.0}");
    s.parse::<u128>()
        .map_or(u64::MAX, |n| u64::try_from(n).unwrap_or(u64::MAX))
}

fn tz_from_opt(s: Option<&str>) -> Option<Tz> {
    s.and_then(|x| Tz::from_str(x).ok())
}

fn parse_ts(ts: &str, tz: Option<Tz>) -> Option<i64> {
    if let Ok(ndt) = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        if let Some(tz) = tz {
            return tz
                .from_local_datetime(&ndt)
                .single()
                .map(|dt| dt.timestamp());
        }
        return Some(ndt.and_utc().timestamp());
    }
    if let Ok(nd) = NaiveDate::parse_from_str(ts, "%Y-%m-%d") {
        let ndt = nd.and_hms_opt(0, 0, 0)?;
        if let Some(tz) = tz {
            return tz
                .from_local_datetime(&ndt)
                .single()
                .map(|dt| dt.timestamp());
        }
        return Some(ndt.and_utc().timestamp());
    }
    None
}

fn usd_money(v: f64) -> Money {
    Money::from_canonical_str(&v.to_string(), Currency::Iso(borsa_core::IsoCurrency::USD)).unwrap()
}

pub fn map_timeseries_to_history(ts: &av::stock_time::TimeSeries) -> HistoryResponse {
    let tz = tz_from_opt(Some(ts.time_zone()));
    let mut candles: Vec<Candle> = Vec::with_capacity(ts.data().len());
    let mut actions = Vec::new();

    for d in ts.data() {
        if let Some(ts_sec) = parse_ts(d.time(), tz) {
            candles.push(Candle {
                ts: Utc.timestamp_opt(ts_sec, 0).unwrap(),
                open: usd_money(d.open()),
                high: usd_money(d.high()),
                low: usd_money(d.low()),
                close: usd_money(d.adjusted().unwrap_or_else(|| d.close())),
                close_unadj: None,
                volume: Some(d.volume()),
            });
            if let Some(div) = d.dividend()
                && div > 0.0
            {
                actions.push(borsa_core::Action::Dividend {
                    ts: Utc.timestamp_opt(ts_sec, 0).unwrap(),
                    amount: usd_money(div),
                });
            }
        }
    }

    candles.sort_by_key(|c| c.ts);
    HistoryResponse {
        candles,
        actions,
        adjusted: ts
            .data()
            .iter()
            .any(|d| d.adjusted().is_some() || d.dividend().is_some()),
        meta: Some(HistoryMeta {
            timezone: tz_from_opt(Some(ts.time_zone())),
            utc_offset_seconds: None,
        }),
    }
}

pub fn map_forex_to_history(fx: &av::forex::Forex) -> HistoryResponse {
    let tz = tz_from_opt(Some(fx.time_zone()));
    let mut candles: Vec<Candle> = Vec::with_capacity(fx.data().len());

    for d in fx.data() {
        if let Some(ts) = parse_ts(d.time(), tz) {
            candles.push(Candle {
                ts: Utc.timestamp_opt(ts, 0).unwrap(),
                open: usd_money(d.open()),
                high: usd_money(d.high()),
                low: usd_money(d.low()),
                close: usd_money(d.close()),
                close_unadj: None,
                volume: None,
            });
        }
    }
    candles.sort_by_key(|c| c.ts);

    HistoryResponse {
        candles,
        actions: vec![],
        adjusted: false,
        meta: Some(HistoryMeta {
            timezone: tz_from_opt(Some(fx.time_zone())),
            utc_offset_seconds: None,
        }),
    }
}

pub fn map_crypto_to_history(c: &av::crypto::Crypto) -> HistoryResponse {
    let tz = tz_from_opt(Some(c.time_zone()));
    let mut candles: Vec<Candle> = Vec::with_capacity(c.data().len());

    for d in c.data() {
        if let Some(ts) = parse_ts(d.time(), tz) {
            let vol_u64: u64 = round_non_negative_f64_to_u64_saturating(d.volume());
            candles.push(Candle {
                ts: Utc.timestamp_opt(ts, 0).unwrap(),
                open: usd_money(d.open()),
                high: usd_money(d.high()),
                low: usd_money(d.low()),
                close: usd_money(d.close()),
                close_unadj: None,
                volume: Some(vol_u64),
            });
        }
    }
    candles.sort_by_key(|c| c.ts);

    HistoryResponse {
        candles,
        actions: vec![],
        adjusted: false,
        meta: Some(HistoryMeta {
            timezone: tz_from_opt(Some(c.time_zone())),
            utc_offset_seconds: None,
        }),
    }
}

pub fn map_kind_from_search_type(t: &str) -> AssetKind {
    match t.trim().to_ascii_uppercase().as_str() {
        "ETF" | "MUTUAL FUND" | "FUND" => AssetKind::Fund,
        "INDEX" => AssetKind::Index,
        "CURRENCY" | "FOREX" => AssetKind::Forex,
        "CRYPTOCURRENCY" | "CRYPTO" | "DIGITAL CURRENCY" => AssetKind::Crypto,
        _ => AssetKind::Equity,
    }
}
