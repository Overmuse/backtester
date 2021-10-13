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
        let mut file = OpenOptions::new().write(true).open(path)?;
        let bytes = serde_json::to_vec(&data)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    async fn load_data(&self, meta: &Meta) -> Result<MarketData, Error> {
        if self.is_cache_valid(meta) {
            let mut path = self.dir.clone();
            path.push("data.json");
            let bytes = std::fs::read(path)?;
            let prices = serde_json::from_slice(&bytes)?;
            Ok(prices)
        } else {
            let mut meta_path = self.dir.clone();
            meta_path.push("meta.json");
            let mut file = OpenOptions::new().write(true).open(meta_path)?;
            let bytes = serde_json::to_vec(meta)?;
            file.write_all(&bytes)?;
            let data = self.data_provider().download_data(meta).await?;
            self.save_data(&data)?;
            Ok(data)
        }
    }
}
