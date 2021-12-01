use chrono::{DateTime, Utc};

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
pub(crate) struct Clock {
    idx: usize,
    market_state: MarketState,
    timestamps: Vec<DateTime<Utc>>,
}

impl Clock {
    pub fn new(timestamps: Vec<DateTime<Utc>>) -> Self {
        Self {
            idx: 0,
            market_state: MarketState::PreOpen,
            timestamps,
        }
    }

    pub fn ticks_remaining(&self) -> usize {
        let state_ticks = match self.state() {
            MarketState::PreOpen => 4,
            MarketState::Opening => 3,
            MarketState::Open => 2,
            MarketState::Closing => 1,
            MarketState::Closed => 0,
        };
        self.timestamps.len() - self.idx + state_ticks
    }

    pub fn previous_datetime(&self) -> Option<&DateTime<Utc>> {
        if self.idx == 0 {
            None
        } else {
            self.timestamps.get(self.idx - 1)
        }
    }

    pub fn datetime(&self) -> Option<&DateTime<Utc>> {
        self.timestamps.get(self.idx)
    }

    pub fn next_datetime(&self) -> Option<&DateTime<Utc>> {
        self.timestamps.get(self.idx + 1)
    }

    pub fn state(&self) -> MarketState {
        self.market_state
    }

    pub fn is_done(&self) -> bool {
        (self.idx == (self.timestamps.len() - 1)) && self.state() == MarketState::Closed
    }

    pub fn is_open(&self) -> bool {
        match self.market_state {
            MarketState::Opening | MarketState::Open | MarketState::Closing => true,
            MarketState::PreOpen | MarketState::Closed => false,
        }
    }

    pub fn warmup(&mut self, periods: usize) {
        self.idx += periods
    }

    pub fn tick(&mut self) {
        let datetime = match self.datetime() {
            Some(datetime) => datetime,
            None => return,
        };
        let next_datetime = self.next_datetime();
        let state = &self.market_state;
        if let Some(next_datetime) = next_datetime {
            if datetime.date() != next_datetime.date() {
                if let MarketState::Closed = state {
                    self.idx += 1
                }
                self.market_state = state.next();
            } else {
                match state {
                    MarketState::PreOpen | MarketState::Opening => self.market_state = state.next(),
                    _ => self.idx += 1,
                }
            }
        } else if let MarketState::Closed = state {
        } else {
            self.market_state = state.next();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn it_can_tell_and_update_time() {
        let mut clock = Clock::new(vec![Utc::now(), Utc::now() + Duration::days(1)]);
        assert!(clock.datetime().is_some());
        assert!(clock.next_datetime().is_some());

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
        assert!(clock.next_datetime().is_none());
    }

    #[test]
    fn it_works_for_intraday_data() {
        let mut clock = Clock::new(vec![
            Utc::now(),
            Utc::now() + Duration::minutes(1),
            Utc::now() + Duration::days(1),
            Utc::now() + Duration::days(1) + Duration::minutes(1),
        ]);
        assert!(clock.datetime().is_some());
        assert!(clock.next_datetime().is_some());

        assert_eq!(clock.state(), MarketState::PreOpen);
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Opening);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Open);
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
        assert!(!clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Opening);
        assert!(clock.is_open());

        clock.tick();
        assert_eq!(clock.state(), MarketState::Open);
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
    }
}
