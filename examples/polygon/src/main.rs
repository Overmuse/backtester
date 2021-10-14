use anyhow::Result;
use backtester::data::{
    downloader::polygon::PolygonDownloader, DataOptions, DataProvider, FileCache,
};
use backtester::{Brokerage, Market, Order, Simulator, Strategy};
use chrono::NaiveDate;
use dotenv::dotenv;
use rust_decimal::Decimal;

struct Strat;

impl Strategy for Strat {
    type Error = anyhow::Error;

    fn at_open(&mut self, brokerage: &mut Brokerage, market: &Market) -> Result<(), Self::Error> {
        let e = market.get_last_price("E");
        let m = market.get_last_price("M");
        if let (Some(e), Some(m)) = (e, m) {
            let order = if e > m {
                Order::new("E", Decimal::ONE)
            } else {
                Order::new("M", Decimal::ONE)
            };
            brokerage.send_order(order);
        }
        let positions = brokerage.get_positions();
        println!("{}", market.datetime().unwrap());
        for position in positions {
            println!("{}", position);
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = PolygonDownloader.file_cache("data");
    let meta = DataOptions::new(
        vec!["E".to_string(), "M".to_string()],
        NaiveDate::from_ymd(2015, 1, 1),
        NaiveDate::from_ymd(2021, 10, 1),
    );
    let data = downloader.download_data(&meta).await?;
    let market = Market::new(data);
    let brokerage = Brokerage::new(Decimal::new(100000, 0), market);
    let mut simulator = Simulator::new(brokerage, Strat);
    simulator.run()
}
