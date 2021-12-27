use crate::utils::nyse_calendar::NyseCalendar;
use crate::Resolution;
use bdays::{HolidayCalendar, HolidayCalendarCache};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, TimeZone};
use chrono_tz::{Tz, US::Eastern};

lazy_static! {
    static ref OPENING_TIME: NaiveTime = NaiveTime::from_hms(9, 30, 0);
    static ref CLOSING_TIME: NaiveTime = NaiveTime::from_hms(16, 0, 0);
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MarketState {
    PreOpen,
    Opening,
    Open,
    Closing,
    Closed,
}

impl MarketState {
    fn next(&self) -> Self {
        match self {
            Self::PreOpen => Self::Opening,
            Self::Opening => Self::Open,
            Self::Open => Self::Closing,
            Self::Closing => Self::Closed,
            Self::Closed => Self::PreOpen,
        }
    }
}

#[derive(Clone)]
struct ClockOptions {
    start: NaiveDate,
    end: NaiveDate,
    warmup: Duration,
    resolution: Resolution,
}

pub struct Clock {
    datetime: DateTime<Tz>,
    market_state: MarketState,
    calendar: HolidayCalendarCache<NaiveDate>,
    options: ClockOptions,
}

impl Clock {
    pub fn new(
        mut start: NaiveDate,
        end: NaiveDate,
        warmup: Duration,
        resolution: Resolution,
    ) -> Self {
        let calendar = HolidayCalendarCache::new(NyseCalendar, start, end);
        if !calendar.is_bday(start) {
            start = calendar.advance_bdays(start, 1);
        }
        let datetime = Eastern
            .from_local_datetime(&start.and_time(*OPENING_TIME))
            .unwrap()
            + warmup;
        let options = ClockOptions {
            start,
            end,
            warmup,
            resolution,
        };
        Self {
            datetime,
            market_state: MarketState::PreOpen,
            calendar,
            options,
        }
    }

    pub fn simulation_periods(&self) -> i32 {
        let days = self
            .calendar
            .bdays(self.datetime.date().naive_local(), self.options.end);
        match self.options.resolution {
            Resolution::Day => days * 5,
            Resolution::Minute => days * 395,
        }
    }

    pub fn is_done(&self) -> bool {
        (self.datetime.date().naive_local() >= self.options.end)
            && self.market_state == MarketState::Closed
    }

    pub fn is_start_of_day(&self) -> bool {
        match self.options.resolution {
            Resolution::Minute => self.datetime.time() == *OPENING_TIME,
            // Since there's only one tick per day, it's always the end of the day
            Resolution::Day => true,
        }
    }

    pub fn is_end_of_day(&self) -> bool {
        // TODO: Fix for early closing time
        match self.options.resolution {
            Resolution::Minute => self.datetime.time() == *CLOSING_TIME,
            // Since there's only one tick per day, it's always the end of the day
            Resolution::Day => true,
        }
    }

    pub fn previous_datetime(&self) -> DateTime<Tz> {
        if self.is_start_of_day() {
            Eastern
                .from_local_datetime(
                    &self
                        .calendar
                        .advance_bdays(self.datetime.date().naive_local(), -1)
                        // TODO: Fix for early closing time
                        .and_time(*CLOSING_TIME),
                )
                .unwrap()
        } else {
            match self.options.resolution {
                Resolution::Minute => self.datetime - Duration::minutes(1),
                Resolution::Day => unreachable!(),
            }
        }
    }

    pub fn datetime(&self) -> DateTime<Tz> {
        self.datetime
    }

    pub fn next_datetime(&self) -> DateTime<Tz> {
        if self.is_end_of_day() {
            Eastern
                .from_local_datetime(
                    &self
                        .calendar
                        .advance_bdays(self.datetime.date().naive_local(), 1)
                        .and_time(*OPENING_TIME),
                )
                .unwrap()
        } else {
            match self.options.resolution {
                Resolution::Minute => self.datetime + Duration::minutes(1),
                Resolution::Day => unreachable!(),
            }
        }
    }

    pub fn state(&self) -> MarketState {
        self.market_state
    }

    pub fn is_open(&self) -> bool {
        match self.market_state {
            MarketState::Opening | MarketState::Open | MarketState::Closing => true,
            MarketState::PreOpen | MarketState::Closed => false,
        }
    }

    pub fn tick(&mut self) {
        if self.is_done() {
            panic!("Market clock ticked after end of backtest");
        }

        let state = &self.market_state;

        if self.is_end_of_day() {
            if let MarketState::Closed = state {
                self.datetime = Eastern
                    .from_local_datetime(
                        &(self
                            .calendar
                            .advance_bdays(self.datetime.date().naive_local(), 1))
                        .and_time(*OPENING_TIME),
                    )
                    .unwrap();
            }
            self.market_state = state.next();
        } else {
            match state {
                MarketState::PreOpen | MarketState::Opening => self.market_state = state.next(),
                _ => match self.options.resolution {
                    Resolution::Minute => self.datetime = self.datetime + Duration::minutes(1),
                    // We should never reach the below as `self.is_end_of_day` should always be
                    // true for daily resolution
                    Resolution::Day => unreachable!(),
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn it_can_tell_and_update_time() {
        let mut clock = Clock::new(
            NaiveDate::from_ymd(2021, 1, 1),
            NaiveDate::from_ymd(2021, 12, 31),
            Duration::zero(),
            Resolution::Day,
        );
        assert_eq!(
            clock.datetime().naive_local(),
            NaiveDate::from_ymd(2021, 1, 5).and_hms(9, 30, 0)
        );
        assert_eq!(
            clock.next_datetime().naive_local(),
            NaiveDate::from_ymd(2021, 1, 6).and_hms(9, 30, 0)
        );

        assert_eq!(clock.state(), MarketState::PreOpen);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Opening);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Open);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Closing);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Closed);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::PreOpen);
    }

    #[test]
    fn it_works_for_intraday_data() {
        let mut clock = Clock::new(
            NaiveDate::from_ymd(2021, 1, 1),
            NaiveDate::from_ymd(2021, 12, 31),
            Duration::zero(),
            Resolution::Minute,
        );
        assert_eq!(
            clock.datetime().naive_local(),
            NaiveDate::from_ymd(2021, 1, 5).and_hms(9, 30, 0)
        );
        assert_eq!(
            clock.next_datetime().naive_local(),
            NaiveDate::from_ymd(2021, 1, 5).and_hms(9, 31, 0)
        );

        assert_eq!(clock.state(), MarketState::PreOpen);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Opening);
        assert!(clock.is_open());

        for _ in 0..391 {
            clock.tick();
            assert_eq!(clock.state(), MarketState::Open);
            assert!(clock.is_open());
        }

        clock.tick();
        assert_eq!(clock.state(), MarketState::Closing);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Closed);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::PreOpen);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Opening);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Open);
        assert!(clock.is_open());
    }
}
