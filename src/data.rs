use crate::context::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use serde_json;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

pub trait DataCache {
    type Error;

    fn is_cache_valid(&self, ctx: Context) -> bool;
    fn save_prices(&self, prices: Vec<f64>) -> Result<(), Self::Error>;
    fn load_prices(&self) -> Result<Vec<f64>, Self::Error>;
}

struct FileDataCache<P, D, S> {
    dir: PathBuf,
    data_provider: DataProvider<P, D, S>,
}

impl<P, D, S> DataCache for FileDataCache<P, D, S> {
    type Error = std::io::Error;
    fn is_cache_valid(&self, _ctx: Context) -> bool {
        todo!()
    }

    fn save_prices(&self, prices: Vec<f64>) -> Result<(), Self::Error> {
        let mut path = self.dir.clone();
        path.push("prices.json");
        let mut file = OpenOptions::new().write(true).open(path)?;
        let bytes = serde_json::to_vec(&prices)?;
        file.write(&bytes)?;
        Ok(())
    }

    fn load_prices(&self) -> Result<Vec<f64>, Self::Error> {
        todo!()
    }
}

pub struct DataProvider<P, D, S> {
    price_downloader: P,
    dividend_downloader: D,
    split_downloader: S,
}

#[async_trait]
pub trait PriceDownloader {
    type Error;
    async fn download_prices(
        tickers: Vec<String>,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<i32>, Self::Error>;
}

pub trait DividendDownloader {
    fn download_dividends();
}

pub trait SplitDownloader {
    fn download_splits();
}

#[cfg(feature = "polygon")]
mod polygon {
    use super::*;
    use ::polygon::{
        errors::Error,
        rest::{Client, GetAggregate},
    };

    pub struct PolygonPriceDownloader;
    #[async_trait]
    impl PriceDownloader for PolygonPriceDownloader {
        type Error = Error;
        async fn download_prices(
            tickers: Vec<String>,
            start: NaiveDate,
            end: NaiveDate,
        ) -> Result<Vec<i32>, Self::Error> {
            let client = Client::from_env()?;
            let queries = tickers
                .iter()
                .map(|ticker| GetAggregate::new(ticker, start, end));
            client
                .send_all(queries)
                .await
                .into_iter()
                .map(|x| x.map(|_| 0i32))
                .collect()
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
