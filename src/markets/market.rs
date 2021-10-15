use crate::data::{Aggregate, MarketData};
use crate::markets::clock::{Clock, MarketState};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

struct Inner {
    clock: Clock,
    data: MarketData,
}

#[derive(Clone)]
pub struct Market {
    inner: Rc<RefCell<Inner>>,
}

impl Market {
    pub fn new(data: MarketData) -> Self {
        let timestamps: BTreeSet<DateTime<Utc>> =
            data.prices.values().flatten().map(|x| *x.0).collect();
        let clock = Clock::new(timestamps.into_iter().collect());
        let inner = Rc::new(RefCell::new(Inner { clock, data }));
        Self { inner }
    }

    pub(crate) fn get_current_price(&self, ticker: &str) -> Option<Decimal> {
        let inner = self.inner.borrow();
        let timeseries = inner.data.prices.get(ticker)?;
        let state = inner.clock.state();
        match state {
            MarketState::PreOpen | MarketState::Closed => None,
            MarketState::Opening => {
                let datetime = inner.clock.datetime()?;
                timeseries.get(datetime).map(|agg| agg.open)
            }
            MarketState::Open | MarketState::Closing => {
                let datetime = inner.clock.datetime()?;
                timeseries.get(datetime).map(|agg| agg.close)
            }
        }
    }

    pub fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        let inner = self.inner.borrow();
        let timeseries = inner.data.prices.get(ticker)?;
        let state = inner.clock.state();
        match state {
            MarketState::PreOpen | MarketState::Opening => {
                let previous_datetime = inner.clock.previous_datetime()?;
                timeseries.get(previous_datetime).map(|agg| agg.close)
            }
            MarketState::Open => {
                let datetime = inner.clock.datetime()?;
                timeseries.get(datetime).map(|agg| agg.open)
            }
            MarketState::Closing | MarketState::Closed => {
                let datetime = inner.clock.datetime()?;
                timeseries.get(datetime).map(|agg| agg.close)
            }
        }
    }

    pub fn get_last_aggregate(&self, ticker: &str) -> Option<Aggregate> {
        let inner = self.inner.borrow();
        let timeseries = inner.data.prices.get(ticker)?;
        let state = inner.clock.state();
        match state {
            MarketState::PreOpen
            | MarketState::Opening
            | MarketState::Open
            | MarketState::Closing => {
                let previous_datetime = inner.clock.previous_datetime()?;
                timeseries.get(previous_datetime).cloned()
            }
            MarketState::Closed => {
                let datetime = inner.clock.datetime()?;
                timeseries.get(datetime).cloned()
            }
        }
    }

    pub fn with_data<F, T>(
        &self,
        ticker: &str,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        f: F,
    ) -> Option<T>
    where
        F: Fn(Vec<&Aggregate>) -> T,
    {
        let inner = self.inner.borrow();
        let timeseries = inner.data.prices.get(ticker)?;
        let data = timeseries.range(start..end).map(|(_, v)| v).collect();
        Some(f(data))
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.inner
            .borrow()
            .clock
            .datetime()
            .expect("Should always be in range")
            .clone()
    }

    pub fn state(&self) -> MarketState {
        self.inner.borrow().clock.state()
    }

    pub(crate) fn is_done(&self) -> bool {
        self.inner.borrow().clock.is_done()
    }

    pub fn is_open(&self) -> bool {
        self.inner.borrow().clock.is_open()
    }

    pub(crate) fn tick(&self) {
        self.inner.borrow_mut().clock.tick()
    }
}
