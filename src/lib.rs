#[macro_use]
extern crate lazy_static;

mod brokerage;
pub mod data;
pub mod finance;
mod markets;
mod options;
mod simulator;
pub mod statistics;
mod strategy;
mod utils;

pub use brokerage::{
    actor::Event,
    handle::Brokerage,
    order::{Order, OrderStatus, OrderType},
};
pub use data::Aggregate;
pub use markets::{clock::MarketState, handle::Market};
pub use options::{Options, Resolution};
pub use simulator::Simulator;
pub use strategy::Strategy;

pub mod prelude {
    pub use crate::data::MarketTimeExt;
    pub use crate::{
        brokerage::{handle::Brokerage, order::Order},
        markets::handle::Market,
        options::{Options, Resolution},
        simulator::Simulator,
        strategy::Strategy,
    };
}
