use crate::brokerage::Brokerage;
use crate::markets::{clock::MarketState, market::Market};
use crate::strategy::Strategy;

pub struct Simulator<S: Strategy> {
    brokerage: Brokerage,
    market: Market,
    strategy: S,
}

impl<S: Strategy> Simulator<S> {
    pub fn new(brokerage: Brokerage, strategy: S) -> Self {
        let market = brokerage.get_market();
        Self {
            brokerage,
            market,
            strategy,
        }
    }
}

impl<S: Strategy> Simulator<S> {
    pub fn run(&mut self) -> Result<(), S::Error> {
        self.strategy.initialize();
        while !self.market.is_done() {
            match self.market.state() {
                MarketState::PreOpen => {
                    self.strategy.before_open(&mut self.brokerage, &self.market)
                }
                MarketState::Opening => self.strategy.at_open(&mut self.brokerage, &self.market),
                MarketState::Open => self.strategy.on_data(&mut self.brokerage, &self.market),
                MarketState::Closing => self.strategy.at_close(&mut self.brokerage, &self.market),
                MarketState::Closed => self.strategy.after_close(&mut self.brokerage, &self.market),
            }?;
            self.market.tick();
        }
        Ok(())
    }
}
