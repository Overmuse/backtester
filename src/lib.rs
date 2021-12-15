#[macro_use]
extern crate lazy_static;

mod brokerage;
pub mod data;
pub mod finance;
mod markets;
mod simulator;
pub mod statistics;
mod strategy;
mod utils;

pub use brokerage::{
    actor::Event,
    handle::Brokerage,
    order::{Order, OrderStatus, OrderType},
};
pub use markets::{clock::MarketState, handle::Market};
pub use simulator::Simulator;
pub use strategy::Strategy;

pub mod prelude {
    pub use crate::data::{provider::DataProvider, DataOptions, MarketTimeExt};
    // pub use crate::data::FileCache;
    pub use crate::{
        brokerage::{handle::Brokerage, order::Order},
        markets::handle::Market,
        simulator::Simulator,
        strategy::Strategy,
    };
}
