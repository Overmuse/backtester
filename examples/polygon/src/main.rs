use anyhow::Result;
use backtester::data::polygon::polygon_downloader;
use backtester::market::Market;
use chrono::{DateTime, NaiveDate, Utc};
use dotenv::dotenv;
use std::collections::BTreeSet;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = polygon_downloader();
    let tickers = vec!["AAPL".to_string(), "TSLA".to_string()];
    let data = downloader
        .download(
            tickers,
            NaiveDate::from_ymd(2021, 1, 1),
            NaiveDate::from_ymd(2021, 10, 1),
        )
        .await?;
    let timestamps: BTreeSet<DateTime<Utc>> = data
        .prices
        .clone()
        .into_values()
        .flatten()
        .into_iter()
        .map(|x| x.0)
        .collect();
    let mut market = Market::new(timestamps.into_iter().collect(), data);
    market.tick();
    market.tick();
    println!("{:?}", market.get_last_price("AAPL"));
    println!("{:?}", market.get_last_price("TSLA"));
    println!("{:?}", market.get_last_price("MSFT"));
    Ok(())
}
