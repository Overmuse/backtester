use crate::brokerage::Brokerage;
use crate::markets::market::Market;
use crate::order::Order;
use chrono::NaiveDate;

pub struct Context {
    tickers: Vec<String>,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

impl Context {
    pub fn new(tickers: Vec<String>, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            tickers,
            start_date,
            end_date,
        }
    }
}

pub trait Strategy {
    type Error;

    fn initialize(&mut self);
    fn before_open(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<Vec<Order>, Self::Error>;
    fn at_open(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<Vec<Order>, Self::Error>;
    fn on_data(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<Vec<Order>, Self::Error>;
    fn at_close(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<Vec<Order>, Self::Error>;
    fn after_close(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<Vec<Order>, Self::Error>;
}
