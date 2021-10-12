use anyhow::Result;
use bdays::{calendars::us::USSettlement, HolidayCalendar, HolidayCalendarCache};
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

    fn trade_dates(&self) -> Vec<NaiveDate> {
        let cal = HolidayCalendarCache::new(USSettlement, self.start_date, self.end_date);
        let mut s = self.start_date;
        let mut out: Vec<NaiveDate> = Vec::new();
        while s <= self.end_date {
            if cal.is_bday(s) {
                out.push(s);
            }
            s = s.succ()
        }
        out
    }
}

struct Data;
struct Trade;

pub trait Strategy {
    fn initialize(&mut self);
    fn before_open(&mut self, ctx: &Context) -> Result<Vec<Trade>>;
    fn at_open(&mut self, data: &Data, ctx: &Context) -> Result<Vec<Trade>>;
    fn on_data(&mut self, data: &Data, ctx: &Context) -> Result<Vec<Trade>>;
    fn at_close(&mut self, data: &Data, ctx: &Context) -> Result<Vec<Trade>>;
    fn after_close(&mut self, data: &Data, ctx: &Context) -> Result<Vec<Trade>>;
}

pub fn run<T: Strategy>(mut strategy: T, ctx: Context) -> Result<()> {
    strategy.initialize();
    let date_range = ctx.trade_dates();
    for date in date_range {
        todo!()
    }
    Ok(())
}
