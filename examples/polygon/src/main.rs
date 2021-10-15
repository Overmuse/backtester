use anyhow::Result;
use backtester::data::downloader::polygon::PolygonDownloader;
use backtester::finance::commission::PerDollarCommission;
use backtester::prelude::*;
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
                Order::new("E", Decimal::ONE).limit_price(e)
            } else {
                Order::new("M", Decimal::ONE).limit_price(m)
            };
            brokerage.send_order(order);
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
        NaiveDate::from_ymd(2021, 10, 1),
        NaiveDate::from_ymd(2021, 10, 14),
    );
    let data = downloader.download_data(&meta).await?;
    let market = Market::new(data);
    let brokerage = Brokerage::new(Decimal::new(100000, 0), market)
        .commission(PerDollarCommission::new(Decimal::ONE));
    let mut simulator = Simulator::new(brokerage, Strat);
    simulator.run()
}
