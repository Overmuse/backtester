use super::{error::Error, provider::DataProvider, DataOptions, MarketData};
use async_trait::async_trait;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

#[async_trait]
pub trait DataCache {
    type DataProvider;

    fn data_provider(&self) -> &Self::DataProvider;
    fn is_cache_valid(&self, meta: &DataOptions) -> bool;
    fn save_data(&self, meta: &DataOptions, data: &MarketData) -> Result<(), Error>;
    async fn load_data(&self, meta: &DataOptions) -> Result<MarketData, Error>;
}

pub trait FileCache {
    fn file_cache<T: Into<PathBuf>>(self, dir: T) -> FileDataCache<Self>
    where
        Self: Sized;
}

impl<T: DataProvider + Send + Sync + 'static> FileCache for T {
    fn file_cache<T2: Into<PathBuf>>(self, dir: T2) -> FileDataCache<Self>
    where
        Self: Sized,
    {
        FileDataCache::new(self, dir.into())
    }
}

#[async_trait]
impl<T> DataProvider for T
where
    T: DataCache + Sync + Send,
    T::DataProvider: DataProvider + Send + Sync,
{
    async fn download_data(&self, meta: &DataOptions) -> Result<MarketData, Error> {
        if self.is_cache_valid(meta) {
            self.load_data(meta).await
        } else {
            let data = self.data_provider().download_data(meta).await?;
            self.save_data(meta, &data)?;
            Ok(data)
        }
    }
}

pub struct FileDataCache<D> {
    dir: PathBuf,
    data_provider: D,
}

impl<D> FileDataCache<D> {
    pub fn new<T>(data_provider: D, dir: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Self {
            data_provider,
            dir: dir.into(),
        }
    }
}

#[async_trait]
impl<T: DataProvider + Send + Sync> DataCache for FileDataCache<T> {
    type DataProvider = T;
    fn data_provider(&self) -> &T {
        &self.data_provider
    }

    fn is_cache_valid(&self, meta: &DataOptions) -> bool {
        let mut path = self.dir.clone();
        path.push("meta.data");
        if path.exists() {
            let bytes = std::fs::read(path);
            if let Ok(bytes) = bytes {
                let cached_meta = rmp_serde::from_slice::<DataOptions>(&bytes);
                if let Ok(cached_meta) = cached_meta {
                    let ticker_check = meta
                        .tickers
                        .iter()
                        .all(|ticker| cached_meta.tickers.contains(ticker));
                    let date_check = cached_meta.start <= meta.start && cached_meta.end >= meta.end;
                    if ticker_check && date_check {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn save_data(&self, meta: &DataOptions, data: &MarketData) -> Result<(), Error> {
        let mut path = self.dir.clone();
        std::fs::create_dir_all(path.clone())?;
        path.push("meta.data");
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        let bytes = rmp_serde::to_vec(meta)?;
        file.write_all(&bytes)?;
        let mut path = self.dir.clone();
        path.push("prices.data");
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        let bytes = rmp_serde::to_vec(&data)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    async fn load_data(&self, meta: &DataOptions) -> Result<MarketData, Error> {
        let mut path = self.dir.clone();
        path.push("prices.data");
        let bytes = std::fs::read(path)?;
        let mut data: MarketData = rmp_serde::from_slice(&bytes)?;
        let t_idx = data
            .timestamps
            .iter()
            .map(|t| t.naive_utc().date() >= meta.start && t.naive_utc().date() <= meta.end);
        let (new_data, new_timestamps) = data
            .data
            .iter()
            .zip(data.timestamps.clone())
            .zip(t_idx)
            .filter_map(|((r, t), i)| if i { Some((r.clone(), t)) } else { None })
            .unzip();
        // TODO: Filter tickers
        //let ticker_idx = data
        //    .tickers
        //    .iter()
        //    .map(|ticker| meta.tickers.contains(ticker));
        //let (new_data, new_tickers) = new_data
        //    .data
        //    .iter()
        //    .zip(data.timestamps.clone())
        //    .zip(t_idx)
        //    .filter_map(|((r, t), i)| if i { Some((r.clone(), t)) } else { None })
        //    .unzip();
        data.data = new_data;
        data.timestamps = new_timestamps;
        Ok(data)
    }
}
