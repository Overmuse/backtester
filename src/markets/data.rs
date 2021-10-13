use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{BTreeMap, HashMap};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Serde(serde_json::Error),
    #[cfg(feature = "polygon")]
    #[error("{0}")]
    Polygon(::polygon::errors::Error),
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
#[cfg(feature = "polygon")]
impl From<::polygon::errors::Error> for Error {
    fn from(e: ::polygon::errors::Error) -> Self {
        Self::Polygon(e)
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

pub struct DataProvider {
    price_downloader: Box<dyn PriceDownloader>,
    dividend_downloader: Box<dyn DividendDownloader>,
    split_downloader: Box<dyn SplitDownloader>,
}

type PriceData = HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>>;

#[derive(Debug)]
pub struct MarketData {
    pub prices: PriceData,
}

impl DataProvider {
    pub async fn download(
        &self,
        tickers: Vec<String>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<MarketData, Error> {
        let prices = self
            .price_downloader
            .download_prices(tickers, start, end)
            .await?;
        Ok(MarketData { prices })
    }
}

#[async_trait]
pub trait PriceDownloader {
    async fn download_prices(
        &self,
        tickers: Vec<String>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<PriceData, Error>;
}

pub trait DividendDownloader {
    fn download_dividends(&self);
}

pub trait SplitDownloader {
    fn download_splits(&self);
}

pub trait DataCache {
    fn is_cache_valid(&self) -> bool;
    fn save_prices(&self, prices: HashMap<String, Vec<Aggregate>>) -> Result<(), Error>;
    fn load_prices(&self) -> Result<HashMap<String, Vec<Aggregate>>, Error>;
}

struct FileDataCache {
    dir: PathBuf,
    data_provider: DataProvider,
}

impl DataCache for FileDataCache {
    fn is_cache_valid(&self) -> bool {
        todo!()
    }

    fn save_prices(&self, prices: HashMap<String, Vec<Aggregate>>) -> Result<(), Error> {
        let mut path = self.dir.clone();
        path.push("prices.json");
        let mut file = OpenOptions::new().write(true).open(path)?;
        let bytes = serde_json::to_vec(&prices)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    fn load_prices(&self) -> Result<HashMap<String, Vec<Aggregate>>, Error> {
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
    use ::polygon::rest::{Aggregate as PolygonAggregate, AggregateWrapper, Client, GetAggregate};

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
        async fn download_prices(
            &self,
            tickers: Vec<String>,
            start: NaiveDate,
            end: NaiveDate,
        ) -> Result<PriceData, Error> {
            let client = Client::from_env()?;
            let queries = tickers
                .iter()
                .map(|ticker| GetAggregate::new(ticker, start, end));
            let wrappers: Result<Vec<AggregateWrapper>, Error> = client
                .send_all(queries)
                .await
                .into_iter()
                .map(|x| x.map_err(From::from))
                .collect();
            let data: HashMap<String, BTreeMap<DateTime<Utc>, Aggregate>> = wrappers?
                .into_iter()
                .map(|w| (w.ticker, w.results.unwrap_or_default()))
                .map(|(ticker, data)| {
                    (
                        ticker,
                        data.into_iter()
                            .map(|agg| (agg.t, From::from(agg)))
                            .collect(),
                    )
                })
                .collect();
            Ok(data)
        }
    }

    pub struct PolygonDividendDownloader;
    impl DividendDownloader for PolygonDividendDownloader {
        fn download_dividends(&self) {}
    }

    pub struct PolygonSplitDownloader;
    impl SplitDownloader for PolygonSplitDownloader {
        fn download_splits(&self) {}
    }

    pub fn polygon_downloader() -> DataProvider {
        DataProvider {
            price_downloader: Box::new(PolygonPriceDownloader),
            dividend_downloader: Box::new(PolygonDividendDownloader),
            split_downloader: Box::new(PolygonSplitDownloader),
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
