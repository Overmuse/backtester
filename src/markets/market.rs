use crate::data::MarketData;
use crate::markets::clock::{Clock, MarketState};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
//pub enum Resolution {
//    Minute,
//    Day,
//}

struct Inner {
    clock: Clock,
    data: MarketData,
}

#[derive(Clone)]
pub struct Market {
    //resolution: Resolution,
    inner: Rc<RefCell<Inner>>,
}

impl Market {
    pub fn new(
        //resolution: Resolution,
        data: MarketData,
    ) -> Self {
        let timestamps: BTreeSet<DateTime<Utc>> =
            data.prices.values().flatten().map(|x| *x.0).collect();
        let clock = Clock::new(timestamps.into_iter().collect());
        let inner = Rc::new(RefCell::new(Inner { clock, data }));
        Self { inner }
    }

    pub fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        let inner = self.inner.borrow();
        let timeseries = inner.data.prices.get(ticker)?;
        let state = self.inner.borrow().clock.state();
        let datetime = inner.clock.datetime()?;
        match state {
            MarketState::PreOpen | MarketState::Opening => {
                let last_dt = timeseries.keys().rfind(|&t| t < datetime)?;
                timeseries.get(last_dt).map(|agg| agg.close)
            }
            MarketState::Open => timeseries
                .get(self.inner.borrow().clock.datetime()?)
                .map(|agg| agg.open),
            MarketState::Closing | MarketState::Closed => timeseries
                .get(self.inner.borrow().clock.datetime()?)
                .map(|agg| agg.close),
        }
    }

    pub fn state(&self) -> MarketState {
        self.inner.borrow().clock.state()
    }

    pub fn is_done(&self) -> bool {
        self.inner.borrow().clock.is_done()
    }

    pub fn tick(&self) {
        self.inner.borrow_mut().clock.tick()
    }
}
