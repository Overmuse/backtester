#[cfg(feature = "polygon")]
pub mod polygon;

use crate::data::{error::Error, Aggregate, Resolution};
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait DataProvider {
    async fn download_data(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
        resolution: Resolution,
    ) -> Result<Vec<Aggregate>, Error>;
}
