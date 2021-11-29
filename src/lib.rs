mod brokerage;
pub mod data;
pub mod finance;
mod markets;
mod simulator;
pub mod statistics;
mod strategy;

pub use brokerage::{
    order::{Order, OrderType},
    Brokerage, OrderStatus,
};
pub use markets::{clock::MarketState, market::Market};
pub use simulator::Simulator;
pub use strategy::Strategy;

pub mod prelude {
    pub use crate::data::{DataOptions, DataProvider, FileCache, MarketTimeExt};
    pub use crate::{
        brokerage::{order::Order, Brokerage},
        markets::market::Market,
        simulator::Simulator,
        strategy::Strategy,
    };
}
