use super::{DataProvider, Error, MarketData, Meta};
use async_trait::async_trait;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

#[async_trait]
pub trait DataCache {
    fn data_provider(&self) -> &Box<dyn DataProvider + Sync + Send>;
    fn is_cache_valid(&self, meta: &Meta) -> bool;
    fn save_data(&self, data: &MarketData) -> Result<(), Error>;
    async fn load_data(&self, meta: &Meta) -> Result<MarketData, Error>;
}

pub trait FileCache {
    fn file_cache<T: Into<PathBuf>>(self, dir: T) -> FileDataCache;
}

impl<T: DataProvider + Send + Sync + 'static> FileCache for T {
    fn file_cache<T2: Into<PathBuf>>(self, dir: T2) -> FileDataCache {
        FileDataCache::new(Box::new(self), dir.into())
    }
}

#[async_trait]
impl<T> DataProvider for T
where
    T: DataCache + Sync + Send,
{
    async fn download_data(&self, meta: &Meta) -> Result<MarketData, Error> {
        self.load_data(meta).await
    }
}

pub struct FileDataCache {
    dir: PathBuf,
    data_provider: Box<dyn DataProvider + Sync + Send>,
}

impl FileDataCache {
    pub fn new<T>(data_provider: Box<dyn DataProvider + Sync + Send>, dir: T) -> Self
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
impl DataCache for FileDataCache {
    fn data_provider(&self) -> &Box<dyn DataProvider + Sync + Send> {
        &self.data_provider
    }

    fn is_cache_valid(&self, meta: &Meta) -> bool {
        let mut path = self.dir.clone();
        path.push("meta.json");
        if path.exists() {
            let bytes = std::fs::read(path);
            if let Ok(bytes) = bytes {
                let cached_meta = serde_json::from_slice::<Meta>(&bytes);
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

    fn save_data(&self, data: &MarketData) -> Result<(), Error> {
        let mut path = self.dir.clone();
        path.push("data.json");
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        let bytes = serde_json::to_vec(&data)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    async fn load_data(&self, meta: &Meta) -> Result<MarketData, Error> {
        if self.is_cache_valid(meta) {
            let mut path = self.dir.clone();
            path.push("data.json");
            let bytes = std::fs::read(path)?;
            let mut data: MarketData = serde_json::from_slice(&bytes)?;
            data.prices
                .retain(|ticker, _| meta.tickers.contains(ticker));
            data.prices.values_mut().for_each(|timeseries| {
                timeseries.retain(|dt, _| {
                    dt.naive_utc().date() > meta.start && dt.naive_utc().date() <= meta.end
                })
            });
            Ok(data)
        } else {
            let mut meta_path = self.dir.clone();
            std::fs::create_dir_all(meta_path.clone())?;
            meta_path.push("meta.json");
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(meta_path)?;
            let bytes = serde_json::to_vec(meta)?;
            file.write_all(&bytes)?;
            let data = self.data_provider().download_data(meta).await?;
            self.save_data(&data)?;
            Ok(data)
        }
    }
}
