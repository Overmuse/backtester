use anyhow::Result;
use async_trait::async_trait;
use backtester::data::provider::polygon::PolygonDownloader;
use backtester::data::Resolution;
use backtester::prelude::*;
use chrono::NaiveDate;
use dotenv::dotenv;
use rand::prelude::*;
use rust_decimal::Decimal;

struct Strat;

#[async_trait]
impl Strategy for Strat {
    type Error = anyhow::Error;

    async fn at_open(&mut self, brokerage: Brokerage, market: Market) -> Result<(), Self::Error> {
        let e = market.get_last_price("E").await;
        let m = market.get_last_price("M").await;
        let equity = Decimal::new(10000, 0);
        if let (Some(e), Some(m)) = (e, m) {
            let amount = if random::<bool>() { equity } else { -equity };

            let order = if e > m {
                Order::new("E", amount).limit_price(e)
            } else {
                Order::new("M", amount).limit_price(m)
            };
            brokerage.send_order(order).await;
        }
        Ok(())
    }

    async fn at_close(&mut self, brokerage: Brokerage, _: Market) -> Result<(), Self::Error> {
        brokerage.close_positions().await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let downloader = PolygonDownloader;
    let data_options = DataOptions::new(
        vec!["E".to_string(), "M".to_string()],
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveDate::from_ymd(2020, 12, 31),
    )
    .set_resolution(Resolution::Minute);
    let simulator = Simulator::new(Decimal::new(100000, 0), Strat, downloader, data_options);
    simulator.run().await
}
