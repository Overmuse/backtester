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
    verbose: bool,
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
            verbose: false,
        }
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

impl<S: Strategy> Simulator<S> {
    pub fn run(&mut self) -> Result<(), S::Error> {
        self.strategy.initialize();
        while !self.market.is_done() {
            if self.verbose {
                print!("{}", CLEAR_SCREEN);
                print!("{}", self.market.datetime());
            }
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
            if let Err(_) = self
                .statistics
                .record_equity(self.market.datetime(), self.brokerage.get_equity())
            {
                println!(
                    "Suspicious equity value\nDatetime: {date}\nEquity: {equity:.2}\nCash:   {cash:.2}",
                    date = self.market.datetime(),
                    equity = self.brokerage.get_equity(),
                    cash = self.brokerage.get_account().cash,
                );
                println!("Positions:");
                for position in self.brokerage.get_positions() {
                    println!("{}", position)
                }
                println!()
            }
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
        //println!("{}", self.statistics);
        println!("{:?}", self.statistics.equity)
        //println!("{:#?}", self.statistics.event_log)
    }
}
