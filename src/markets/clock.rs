use chrono::{DateTime, Utc};

#[derive(Copy, Clone, Debug)]
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

pub struct Clock {
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

    pub fn datetime(&self) -> Option<&DateTime<Utc>> {
        self.timestamps.get(self.idx)
    }

    pub fn next_datetime(&self) -> Option<&DateTime<Utc>> {
        self.timestamps.get(self.idx + 1)
    }

    fn set_idx(&mut self, idx: usize) {
        self.idx = idx
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
        let datetime = match self.datetime() {
            Some(datetime) => datetime,
            None => return,
        };
        let next_datetime = self.next_datetime();
        if let Some(next_datetime) = next_datetime {
            let state = &self.market_state;
            if datetime.date() != next_datetime.date() {
                if let MarketState::Closed = state {
                    self.idx += 1
                }
                self.market_state = state.next();
            } else {
                match state {
                    MarketState::Opening | MarketState::Open => self.market_state = state.next(),
                    _ => self.idx += 1,
                }
            }
        }
    }
}
