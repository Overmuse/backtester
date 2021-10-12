use crate::brokerage::Brokerage;
use crate::market::Market;
use crate::strategy::Strategy;

struct Simulator<S: Strategy> {
    brokerage: Brokerage,
    market: Market,
    strategy: S,
}
