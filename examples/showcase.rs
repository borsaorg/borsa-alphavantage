use std::error::Error;

use borsa_alphavantage::AvConnector;
use borsa_core::{AssetKind, HistoryRequest, Instrument, Interval, Range, SearchRequest};

// Bring provider traits into scope for method call syntax
use borsa_core::connector::{/*EarningsProvider,*/ HistoryProvider, QuoteProvider, SearchProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env variables (ALPHAVANTAGE_API_KEY)
    let _ = dotenvy::dotenv();
    let api_key = std::env::var("ALPHAVANTAGE_API_KEY")
        .expect("Set ALPHAVANTAGE_API_KEY in a .env file at the repo root");

    // Build the Alpha Vantage connector
    let av = AvConnector::new_with_key(api_key);

    // --- Quote example (AAPL) ---
    println!("--- Quote example (AAPL) ---");
    let aapl = Instrument::from_symbol("AAPL", AssetKind::Equity)?;
    let quote = av.quote(&aapl).await?;
    match quote.price {
        Some(p) => println!("Quote {}: {}", quote.symbol.as_str(), p),
        None => println!("Quote {}: <no price>", quote.symbol.as_str()),
    }

    // --- Search example ("apple", limit 5) ---
    println!("--- Search example ('apple', limit 5) ---");
    let sreq = SearchRequest::builder("apple").limit(5).build()?;
    let sresp = av.search(sreq).await?;
    println!("Search results ({}):", sresp.results.len());
    for r in &sresp.results {
        println!(
            "- {}  kind={:?}  name={}",
            r.symbol.as_str(),
            r.kind,
            r.name.as_deref().unwrap_or("")
        );
    }

    // --- Optional: Forex history example (EUR/USD, daily) ---
    println!("--- Forex history example (EUR/USD, daily) ---");
    let eurusd = Instrument::from_symbol("EUR/USD", AssetKind::Forex)?;
    let fx_hist_req = HistoryRequest::try_from_range(Range::D1, Interval::D1)?;
    let fx_hist = av.history(&eurusd, fx_hist_req).await?;
    println!("Forex history {}: {} candles", eurusd.symbol(), fx_hist.candles.len());

    // Premium Endpoints (Requires a Premium API Key)

    // --- History example (AAPL, daily candles) --- 
    println!("--- Premium history example (AAPL, daily candles) ---");
    let hist_req = HistoryRequest::try_from_range(Range::D1, Interval::D1)?;
    let history = av.history(&aapl, hist_req).await?;
    println!("History {}: {} candles (adjusted={})",
        aapl.symbol(), history.candles.len(), history.adjusted);
    if let Some(last) = history.candles.last() {
        println!("Last close {} at {}", last.close, last.ts);
    }
    
    // Not working as expected, returns an error from the underlying crate.

    // --- Earnings example (AAPL) ---
    /*
    println!("--- Earnings example (AAPL) ---");
    let earnings = av.earnings(&aapl).await?;
    println!(
        "Earnings: yearly={} quarterly_eps={}",
        earnings.yearly.len(),
        earnings.quarterly_eps.len()
    );
    */

    Ok(())
}


