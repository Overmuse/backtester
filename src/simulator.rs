use crate::brokerage::{
    actor::{BrokerageActor, Event},
    handle::Brokerage,
    order::{Order, OrderStatus},
};
use crate::data::{provider::DataProvider, DataOptions};
use crate::markets::{actor::MarketActor, clock::MarketState, handle::Market};
use crate::statistics::Statistics;
use crate::strategy::Strategy;
use chrono::DateTime;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use std::fs::{create_dir_all, remove_file, OpenOptions};
use std::io::Write;
use tracing::trace;

pub struct Simulator<S: Strategy + Send + Sync> {
    brokerage: Brokerage,
    market: Market,
    strategy: S,
    statistics: Statistics,
    data_options: DataOptions,
}

impl<S: Strategy + Send + Sync> Simulator<S> {
    pub fn new<D: 'static + DataProvider + Send + Sync>(
        cash: Decimal,
        strategy: S,
        data_provider: D,
        data_options: DataOptions,
    ) -> Self {
        let market = MarketActor::spawn(data_provider, data_options.clone());
        let brokerage = BrokerageActor::spawn(cash, market.clone());
        let statistics = Statistics::new();
        Self {
            brokerage,
            market,
            strategy,
            statistics,
            data_options,
        }
    }

    pub async fn run(mut self) -> Result<(), S::Error> {
        self.strategy.initialize().await;
        let mut event_listener = self.brokerage.subscribe().await;
        while !self.market.is_done().await {
            let (datetime, state) = futures::join!(self.market.datetime(), self.market.state());
            trace!("{} - {:?}", datetime, state);
            match state {
                MarketState::PreOpen => {
                    self.strategy
                        .before_open(self.brokerage.clone(), self.market.clone())
                        .await
                }
                MarketState::Opening => {
                    self.strategy
                        .at_open(self.brokerage.clone(), self.market.clone())
                        .await
                }
                MarketState::Open => {
                    self.brokerage.reconcile_active_orders().await;
                    self.strategy
                        .during_regular_hours(self.brokerage.clone(), self.market.clone())
                        .await
                }
                MarketState::Closing => {
                    self.strategy
                        .at_close(self.brokerage.clone(), self.market.clone())
                        .await
                }
                MarketState::Closed => {
                    self.brokerage.expire_orders().await;
                    self.strategy
                        .after_close(self.brokerage.clone(), self.market.clone())
                        .await?;
                    Ok(())
                }
            }?;
            while let Ok(event) = event_listener.try_recv() {
                trace!("Event received: {:?}", event);
                self.strategy.on_event(event.clone()).await?;
                self.handle_event(event)
            }
            let equity = self.brokerage.get_equity().await;
            trace!("Equity: {:.2}", equity);
            self.statistics.record_equity(datetime, equity);
            self.market.tick().await;
        }
        self.generate_report();
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        self.statistics.record_event(event.clone());
        match event {
            Event::OrderUpdate {
                status,
                order,
                time,
            } => self.handle_order_update(status, order, time),
            Event::Commission { amount } => self.statistics.increase_commission(amount),
        }
    }

    fn handle_order_update(&mut self, status: OrderStatus, _order: Order, _time: DateTime<Tz>) {
        self.statistics.handle_order(&status)
    }

    pub fn generate_report(self) {
        let outdir = self
            .data_options
            .outdir
            .unwrap_or_else(|| "out".to_string());
        let _ = create_dir_all(outdir.clone());
        let filename = format!("{}/statistics.txt", outdir);
        let _ = remove_file(filename.clone());
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)
            .unwrap();
        write!(file, "{}", self.statistics).unwrap();
        file.flush().unwrap();

        let filename = format!("{}/equity.csv", outdir);
        let _ = remove_file(filename.clone());
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)
            .unwrap();
        let mut wtr = csv::Writer::from_writer(file);
        wtr.write_record(&["datetime", "equity"]).unwrap();
        for (d, e) in self.statistics.equity {
            wtr.write_record(&[d.to_string(), e.to_string()]).unwrap()
        }
        wtr.flush().unwrap();

        let filename = format!("{}/event_log.json", outdir);
        let _ = remove_file(filename.clone());
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)
            .unwrap();
        write!(
            file,
            "{}",
            serde_json::to_string_pretty(&self.statistics.event_log).unwrap()
        )
        .unwrap();
        file.flush().unwrap();
    }
}
