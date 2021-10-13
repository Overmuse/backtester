use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub use cache::FileCache;
pub use error::Error;
pub mod cache;
pub mod downloader;
pub mod error;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataOptions {
    tickers: Vec<String>,
    start: NaiveDate,
    end: NaiveDate,
}

impl DataOptions {
    pub fn new(tickers: Vec<String>, start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            tickers,
            start,
            end,
        }
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

type PriceData = HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketData {
    pub prices: PriceData,
}

#[async_trait]
pub trait DataProvider {
    async fn download_data(&self, meta: &DataOptions) -> Result<MarketData, Error>;
}
