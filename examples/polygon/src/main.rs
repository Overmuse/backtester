use anyhow::Result;
use backtester::markets::{data::polygon::polygon_downloader, market::Market};
use chrono::NaiveDate;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = polygon_downloader();
    let tickers = vec!["AAPL".to_string(), "TSLA".to_string()];
    let data = downloader
        .download(
            tickers,
            NaiveDate::from_ymd(2020, 1, 1),
            NaiveDate::from_ymd(2021, 10, 1),
        )
        .await?;
    let mut market = Market::new(data);
    market.tick();
    market.tick();
    println!("{:?}", market.get_last_price("AAPL"));
    println!("{:?}", market.get_last_price("TSLA"));
    println!("{:?}", market.get_last_price("MSFT"));
    Ok(())
}
