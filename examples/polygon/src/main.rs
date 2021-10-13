use anyhow::Result;
use backtester::data::Meta;
use backtester::data::{
    cache::FileDataCache, downloader::polygon::PolygonDownloader, DataProvider,
};
use backtester::markets::market::Market;
use chrono::NaiveDate;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = PolygonDownloader;
    let cache = FileDataCache::new(Box::new(downloader), "./data");
    let tickers = vec!["AAP".to_string(), "TSLA".to_string()];
    let meta = Meta::new(
        tickers,
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2021, 10, 1),
    );
    let data = cache.download_data(&meta).await?;
    let mut market = Market::new(data);
    market.tick();
    market.tick();
    println!("{:?}", market.get_last_price("AAPL"));
    println!("{:?}", market.get_last_price("TSLA"));
    println!("{:?}", market.get_last_price("MSFT"));
    Ok(())
}
