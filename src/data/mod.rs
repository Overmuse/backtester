use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::US::Eastern;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub use cache::FileCache;
mod cache;
pub mod downloader;
pub mod error;

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Resolution {
    Minute,
    Day,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataOptions {
    tickers: Vec<String>,
    start: NaiveDate,
    end: NaiveDate,
    resolution: Resolution,
}

impl DataOptions {
    pub fn new(tickers: Vec<String>, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            tickers,
            start,
            end,
            resolution: Resolution::Day,
        }
    }

    pub fn resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    pub datetime: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

pub trait MarketTimeExt {
    fn is_regular_hours(&self) -> bool;
    fn is_opening(&self) -> bool;
    fn is_closing(&self) -> bool;
}

impl<T: TimeZone> MarketTimeExt for DateTime<T> {
    fn is_regular_hours(&self) -> bool {
        let zoned = self.with_timezone(&Eastern);
        (zoned.time() >= NaiveTime::from_hms(9, 30, 00))
            && (zoned.time() < NaiveTime::from_hms(16, 00, 00))
    }
    fn is_opening(&self) -> bool {
        self.with_timezone(&Eastern).time() == NaiveTime::from_hms(9, 30, 0)
    }
    fn is_closing(&self) -> bool {
        self.with_timezone(&Eastern).time() == NaiveTime::from_hms(16, 0, 0)
    }
}

type PriceData = HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    pub prices: PriceData,
    pub resolution: Resolution,
}

impl MarketData {
    pub fn normalize_data(&mut self) {
        self.prices.values_mut().for_each(|v| {
            v.retain(|d, _| {
                let dt_tz = d.with_timezone(&Eastern);
                (dt_tz.time() >= NaiveTime::from_hms(9, 30, 0))
                    && (dt_tz.time() < NaiveTime::from_hms(16, 0, 0))
            });
        });
    }
}

#[async_trait]
pub trait DataProvider {
    async fn download_data(&self, meta: &DataOptions) -> Result<MarketData, error::Error>;
}
