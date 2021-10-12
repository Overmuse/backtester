use backtester::data::polygon::polygon_downloader;
use chrono::NaiveDate;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    let downloader = polygon_downloader();
    let tickers = vec!["AAPL".to_string(), "TSLA".to_string()];
    let out = downloader
        .download(
            tickers,
            NaiveDate::from_ymd(2021, 1, 1),
            NaiveDate::from_ymd(2021, 10, 1),
        )
        .await;
    println!("{:?}", out);
}
