use crate::brokerage::{order::Order, Brokerage, Event, OrderStatus};
use crate::markets::{clock::MarketState, market::Market};
use crate::statistics::Statistics;
use crate::strategy::Strategy;
use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use log::trace;
use std::sync::mpsc::Receiver;

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
        let progress = ProgressBar::new(self.market.ticks_remaining() as u64)
            .with_message("Backtesting")
            .with_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .progress_chars("##-"),
            );
        while !self.market.is_done() {
            let datetime = self.market.datetime();
            let state = self.market.state();
            trace!("{} - {:?}", datetime, state);
            match state {
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
                MarketState::Closed => {
                    self.brokerage.expire_orders();
                    self.strategy.after_close(&mut self.brokerage, &self.market)
                }
            }?;
            while let Ok(event) = self.event_listener.try_recv() {
                trace!("Event received: {:?}", event);
                self.strategy.on_event(event.clone())?;
                self.handle_event(event)
            }
            let equity = self.brokerage.get_equity();
            trace!("Equity: {:.2}", equity);
            self.statistics.record_equity(datetime, equity);
            self.market.tick();
            progress.inc(1);
        }
        progress.finish();
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
        println!("{}", self.statistics);
        //println!("{:?}", self.statistics.equity)
        //println!("{:#?}", self.statistics.event_log)
    }
}
