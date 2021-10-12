use crate::context::Context;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    pub datetime: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

pub struct DataProvider<P, D, S> {
    price_downloader: P,
    dividend_downloader: D,
    split_downloader: S,
}

impl<P: PriceDownloader, D, S> DataProvider<P, D, S> {
    pub async fn download(
        &self,
        tickers: Vec<String>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<HashMap<String, Vec<Aggregate>>, P::Error> {
        let prices = self
            .price_downloader
            .download_prices(tickers, start, end)
            .await?;
        Ok(prices)
    }
}

#[async_trait]
pub trait PriceDownloader {
    type Error;
    async fn download_prices(
        &self,
        tickers: Vec<String>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<HashMap<String, Vec<Aggregate>>, Self::Error>;
}

pub trait DividendDownloader {
    fn download_dividends();
}

pub trait SplitDownloader {
    fn download_splits();
}

pub trait DataCache {
    type Error;

    fn is_cache_valid(&self, ctx: Context) -> bool;
    fn save_prices(&self, prices: HashMap<String, Vec<Aggregate>>) -> Result<(), Self::Error>;
    fn load_prices(&self) -> Result<HashMap<String, Vec<Aggregate>>, Self::Error>;
}

struct FileDataCache<P, D, S> {
    dir: PathBuf,
    data_provider: DataProvider<P, D, S>,
}

enum Error {
    Io(std::io::Error),
    Serde(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl<P, D, S> DataCache for FileDataCache<P, D, S> {
    type Error = Error;
    fn is_cache_valid(&self, _ctx: Context) -> bool {
        todo!()
    }

    fn save_prices(&self, prices: HashMap<String, Vec<Aggregate>>) -> Result<(), Self::Error> {
        let mut path = self.dir.clone();
        path.push("prices.json");
        let mut file = OpenOptions::new().write(true).open(path)?;
        let bytes = serde_json::to_vec(&prices)?;
        file.write(&bytes)?;
        Ok(())
    }

    fn load_prices(&self) -> Result<HashMap<String, Vec<Aggregate>>, Self::Error> {
        let mut path = self.dir.clone();
        path.push("prices.json");
        let bytes = std::fs::read(path)?;
        let prices = serde_json::from_slice(&bytes)?;
        Ok(prices)
    }
}

#[cfg(feature = "polygon")]
pub mod polygon {
    use super::*;
    use ::polygon::{
        errors::Error,
        rest::{Aggregate as PolygonAggregate, AggregateWrapper, Client, GetAggregate},
    };

    pub struct PolygonPriceDownloader;

    impl From<PolygonAggregate> for Aggregate {
        fn from(p: PolygonAggregate) -> Aggregate {
            Aggregate {
                datetime: p.t,
                open: p.o,
                high: p.h,
                low: p.l,
                close: p.c,
                volume: p.v,
            }
        }
    }

    #[async_trait]
    impl PriceDownloader for PolygonPriceDownloader {
        type Error = Error;
        async fn download_prices(
            &self,
            tickers: Vec<String>,
            start: NaiveDate,
            end: NaiveDate,
        ) -> Result<HashMap<String, Vec<Aggregate>>, Self::Error> {
            let client = Client::from_env()?;
            let queries = tickers
                .iter()
                .map(|ticker| GetAggregate::new(ticker, start, end));
            let wrappers: Result<Vec<AggregateWrapper>, Self::Error> =
                client.send_all(queries).await.into_iter().collect();
            let data: HashMap<String, Vec<Aggregate>> = wrappers?
                .into_iter()
                .map(|w| (w.ticker, w.results.unwrap_or(Vec::new())))
                .map(|(ticker, data)| (ticker, data.into_iter().map(From::from).collect()))
                .collect();
            Ok(data)
        }
    }

    pub struct PolygonDividendDownloader;
    impl DividendDownloader for PolygonDividendDownloader {
        fn download_dividends() {}
    }

    pub struct PolygonSplitDownloader;
    impl SplitDownloader for PolygonSplitDownloader {
        fn download_splits() {}
    }

    pub fn polygon_downloader(
    ) -> DataProvider<PolygonPriceDownloader, PolygonDividendDownloader, PolygonSplitDownloader>
    {
        DataProvider {
            price_downloader: PolygonPriceDownloader,
            dividend_downloader: PolygonDividendDownloader,
            split_downloader: PolygonSplitDownloader,
        }
    }
}

#[cfg(feature = "iex")]
mod iex {
    use super::*;

    pub struct IexPriceDownloader;
    impl PriceDownloader for IexPriceDownloader {
        fn download_prices() {}
    }

    pub struct IexDividendDownloader;
    impl DividendDownloader for IexDividendDownloader {
        fn download_dividends() {}
    }

    pub struct IexSplitDownloader;
    impl SplitDownloader for IexSplitDownloader {
        fn download_splits() {}
    }

    pub fn iex_downloader(
    ) -> DataProvider<IexPriceDownloader, IexDividendDownloader, IexSplitDownloader> {
        DataProvider {
            price_downloader: IexPriceDownloader,
            dividend_downloader: IexDividendDownloader,
            split_downloader: IexSplitDownloader,
        }
    }
}
