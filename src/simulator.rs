use crate::brokerage::{order::Order, Brokerage, Event, OrderStatus};
use crate::markets::{clock::MarketState, market::Market};
use crate::statistics::Statistics;
use crate::strategy::Strategy;
use chrono::{DateTime, Utc};
use std::sync::mpsc::Receiver;

const CLEAR_SCREEN: &str = "\x1B[2J\x1B[1;1H";

pub struct Simulator<S: Strategy> {
    brokerage: Brokerage,
    market: Market,
    strategy: S,
    statistics: Statistics,
    event_listener: Receiver<Event>,
}

impl<S: Strategy> Simulator<S> {
    pub fn new(mut brokerage: Brokerage, strategy: S) -> Self {
        let market = brokerage.get_market();
        let event_listener = brokerage.subscribe();
        let statistics = Statistics::new();
        Self {
            brokerage,
            market,
            strategy,
            statistics,
            event_listener,
        }
    }
}

impl<S: Strategy> Simulator<S> {
    pub fn run(&mut self) -> Result<(), S::Error> {
        self.strategy.initialize();
        while !self.market.is_done() {
            print!("{}", CLEAR_SCREEN);
            println!("{}", self.market.datetime());
            match self.market.state() {
                MarketState::PreOpen => {
                    self.strategy.before_open(&mut self.brokerage, &self.market)
                }
                MarketState::Opening => self.strategy.at_open(&mut self.brokerage, &self.market),
                MarketState::Open => {
                    self.brokerage.reconcile_active_orders();
                    self.strategy
                        .during_regular_hours(&mut self.brokerage, &self.market)
                }
                MarketState::Closing => self.strategy.at_close(&mut self.brokerage, &self.market),
                MarketState::Closed => self.strategy.after_close(&mut self.brokerage, &self.market),
            }?;
            while let Ok(event) = self.event_listener.try_recv() {
                self.strategy.on_event(event.clone())?;
                self.handle_event(event)
            }
            self.statistics.record_equity(self.brokerage.get_equity());
            self.market.tick();
        }
        self.generate_report();
        Ok(())
    }

    fn handle_event(&mut self, event: Event) {
        self.statistics.record_event(event.clone());
        match event {
            Event::OrderUpdate {
                status,
                order,
                time,
            } => self.handle_order_update(status, order, time),
            Event::Commission { amount } => self.statistics.increase_commission(amount),
        }
    }

    fn handle_order_update(&mut self, status: OrderStatus, _order: Order, _time: DateTime<Utc>) {
        self.statistics.handle_order(&status)
    }

    pub fn generate_report(&self) {
        println!("{}", self.statistics)
    }
}
