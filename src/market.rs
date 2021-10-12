use crate::clock::{Clock, MarketState};
use crate::data::MarketData;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
//pub enum Resolution {
//    Minute,
//    Day,
//}

pub struct Market {
    //resolution: Resolution,
    clock: Clock,
    data: MarketData,
}

impl Market {
    pub fn new(
        //resolution: Resolution,
        timestamps: Vec<DateTime<Utc>>,
        data: MarketData,
    ) -> Self {
        let clock = Clock::new(timestamps);
        Self {
            //resolution,
            clock,
            data,
        }
    }

    pub fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        let timeseries = self.data.prices.get(ticker)?;
        let state = self.clock.state();
        let datetime = self.clock.datetime()?;
        match state {
            MarketState::PreOpen | MarketState::Opening => {
                let last_dt = timeseries.keys().rfind(|&t| t < datetime)?;
                timeseries.get(last_dt).map(|agg| agg.close)
            }
            MarketState::Open => timeseries.get(self.clock.datetime()?).map(|agg| agg.open),
            MarketState::Closing | MarketState::Closed => {
                timeseries.get(self.clock.datetime()?).map(|agg| agg.close)
            }
        }
    }

    pub fn tick(&mut self) {
        self.clock.tick()
    }
}
