use crate::data::{Aggregate, MarketData, MarketTimeExt};
use crate::markets::clock::{Clock, MarketState};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
struct MarketDataCache {
    last: HashMap<String, Aggregate>,
    current: HashMap<String, Aggregate>,
    last_open: HashMap<String, Decimal>,
    last_close: HashMap<String, Decimal>,
}

impl MarketDataCache {
    fn update(&mut self, ticker: &str, aggregate: &Aggregate) {
        if aggregate.datetime.is_opening() {
            self.last_open.insert(ticker.to_string(), aggregate.open);
        } else if aggregate.datetime.is_closing() {
            self.last_close.insert(ticker.to_string(), aggregate.close);
        }
        let last = self.current.insert(ticker.to_string(), aggregate.clone());
        if let Some(last) = last {
            self.last.insert(ticker.to_string(), last);
        }
    }

    fn current(&self, ticker: &str) -> Option<Aggregate> {
        self.current.get(ticker).cloned()
    }

    fn last(&self, ticker: &str) -> Option<Aggregate> {
        self.last.get(ticker).cloned()
    }

    fn last_open(&self, ticker: &str) -> Option<Decimal> {
        self.last_open.get(ticker).copied()
    }

    fn last_close(&self, ticker: &str) -> Option<Decimal> {
        self.last_close.get(ticker).copied()
    }
}

#[derive(Clone)]
pub struct Market {
    clock: Rc<RefCell<Clock>>,
    cache: Rc<RefCell<MarketDataCache>>,
    data: Rc<RefCell<MarketData>>,
}

impl Market {
    pub fn new(data: MarketData) -> Self {
        let timestamps = data.timestamps().to_vec();
        let clock = Rc::new(RefCell::new(Clock::new(timestamps)));
        let cache = Rc::new(RefCell::new(MarketDataCache::default()));
        let data = Rc::new(RefCell::new(data));
        Self { clock, cache, data }
    }

    pub fn warmup(&mut self, periods: usize) {
        self.clock.borrow_mut().warmup(periods)
    }

    pub fn ticks_remaining(&self) -> usize {
        self.clock.borrow().ticks_remaining()
    }

    pub(crate) fn get_current_price(&self, ticker: &str) -> Option<Decimal> {
        self.cache.borrow().current(ticker).map(|a| a.close)
    }

    // TODO: Update all below to use the cache

    pub fn get_open(&self, ticker: &str) -> Option<Decimal> {
        self.cache.borrow().last_open(ticker)
    }

    pub fn get_close(&self, ticker: &str) -> Option<Decimal> {
        self.cache.borrow().last_close(ticker)
    }

    pub fn get_last_price(&self, ticker: &str) -> Option<Decimal> {
        self.cache.borrow().last(ticker).map(|a| a.close)
    }

    pub fn get_last_aggregate(&self, ticker: &str) -> Option<Aggregate> {
        self.cache.borrow().last(ticker)
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
        let data = self.data.borrow();
        let timeseries = data.get_timeseries(ticker, Some(*start), Some(*end));
        let data = timeseries.filter_map(|(_, a)| a.as_ref()).collect();
        Some(f(data))
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        *self
            .clock
            .borrow()
            .datetime()
            .expect("Should always be in range")
    }

    pub fn state(&self) -> MarketState {
        self.clock.borrow().state()
    }

    pub(crate) fn is_done(&self) -> bool {
        self.clock.borrow().is_done()
    }

    pub fn is_open(&self) -> bool {
        self.clock.borrow().is_open()
    }

    pub fn previous_datetime(&self) -> Option<DateTime<Utc>> {
        self.clock.borrow().previous_datetime().copied()
    }

    pub(crate) fn tick(&self) {
        self.clock.borrow_mut().tick();
        let data = self.data.borrow();
        let data: Vec<(&str, &Option<Aggregate>)> = data
            .get_timestamp(*self.clock.borrow().datetime().unwrap())
            .collect();
        for (ticker, agg) in data {
            if let Some(agg) = agg {
                self.cache.borrow_mut().update(ticker, agg)
            }
        }
    }
}
