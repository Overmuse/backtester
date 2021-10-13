use anyhow::Result;
use backtester::data::{
    downloader::polygon::PolygonDownloader, DataOptions, DataProvider, FileCache,
};
use backtester::markets::market::Market;
use chrono::NaiveDate;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = PolygonDownloader.file_cache("data");
    let tickers = vec!["AAPL".to_string(), "TSLA".to_string()];
    let meta = DataOptions::new(
        tickers,
        NaiveDate::from_ymd(2015, 1, 1),
        NaiveDate::from_ymd(2021, 10, 1),
    );
    let data = downloader.download_data(&meta).await?;
    let mut market = Market::new(data);
    market.tick();
    market.tick();
    println!("{:?}", market.get_last_price("AAPL"));
    println!("{:?}", market.get_last_price("TSLA"));
    println!("{:?}", market.get_last_price("MSFT"));
    Ok(())
}
