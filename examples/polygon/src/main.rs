use anyhow::{anyhow, Result};
use backtester::brokerage::brokerage::Brokerage;
use backtester::brokerage::order::Order;
use backtester::data::{
    downloader::polygon::PolygonDownloader, DataOptions, DataProvider, FileCache,
};
use backtester::markets::market::Market;
use backtester::simulator::Simulator;
use backtester::strategy::Strategy;
use chrono::NaiveDate;
use dotenv::dotenv;
use rust_decimal::Decimal;

struct Strat;

impl Strategy for Strat {
    type Error = String;

    fn at_open(&mut self, brokerage: &mut Brokerage, _market: &Market) -> Result<(), Self::Error> {
        let positions = brokerage.get_positions();
        for position in positions {
            println!("{}", position);
        }
        let order = Order::new("AAPL".to_string(), Decimal::ONE);
        brokerage.send_order(order);
        let order = Order::new("M".to_string(), Decimal::ONE);
        brokerage.send_order(order);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = PolygonDownloader.file_cache("data");
    let meta = DataOptions::new(
        vec!["AAPL".to_string(), "M".to_string()],
        NaiveDate::from_ymd(2015, 1, 1),
        NaiveDate::from_ymd(2021, 10, 1),
    );
    let data = downloader.download_data(&meta).await?;
    let market = Market::new(data);
    let brokerage = Brokerage::new(Decimal::new(100000, 0), market.clone());
    let strategy = Strat;
    let mut simulator = Simulator::new(brokerage, market, strategy);
    simulator.run().map_err(|s| anyhow!("{}", s))
}
