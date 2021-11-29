use anyhow::Result;
use backtester::data::downloader::polygon::PolygonDownloader;
use backtester::data::Resolution;
use backtester::finance::commission::PerDollarCommission;
use backtester::prelude::*;
use chrono::NaiveDate;
use dotenv::dotenv;
use rand::prelude::*;
use rust_decimal::Decimal;

struct Strat;

impl Strategy for Strat {
    type Error = anyhow::Error;

    fn at_open(&mut self, brokerage: &mut Brokerage, market: &Market) -> Result<(), Self::Error> {
        let e = market.get_last_price("E");
        let m = market.get_last_price("M");
        if let (Some(e), Some(m)) = (e, m) {
            let amount = if random::<bool>() {
                Decimal::ONE
            } else {
                -Decimal::ONE
            };

            let order = if e > m {
                Order::new("E", amount).limit_price(e)
            } else {
                Order::new("M", amount).limit_price(m)
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
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2020, 12, 31),
    )
    .resolution(Resolution::Minute);
    let mut data = downloader.download_data(&meta).await?;
    data.normalize_data();
    let market = Market::new(data);
    let brokerage = Brokerage::new(Decimal::new(100000, 0), market)
        .commission(PerDollarCommission::new(Decimal::new(1, 3)));
    let mut simulator = Simulator::new(brokerage, Strat).verbose(true);
    simulator.run()
}
