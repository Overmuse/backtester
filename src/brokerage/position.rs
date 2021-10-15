use chrono::{DateTime, Utc};
use num_traits::Signed;
use rust_decimal::prelude::*;
use std::collections::VecDeque;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Lot {
    pub fill_time: DateTime<Utc>,
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub ticker: String,
    lots: VecDeque<Lot>,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{: >5}: {: >7} @ {}",
            self.ticker,
            self.quantity(),
            self.average_price().unwrap_or(Decimal::ZERO)
        )
    }
}

impl Position {
    pub fn new(ticker: String, lot: Lot) -> Self {
        let mut lots = VecDeque::new();
        lots.push_back(lot);
        Self { ticker, lots }
    }

    pub fn add_lot(&mut self, new_lot: Lot) {
        let current_quantity = self.quantity();
        if current_quantity.signum() * new_lot.quantity.signum() >= Decimal::ZERO {
            // Same sign or zero, so we can just accumulate the position
            self.lots.push_back(new_lot)
        } else {
            // Need to dispose of lot using FIFO logic
            let mut unaccounted = new_lot.quantity;
            while unaccounted != Decimal::ZERO {
                let first = self.lots.pop_front();
                match first {
                    Some(mut fifo_lot) => {
                        if fifo_lot.quantity.abs() > unaccounted.abs() {
                            fifo_lot.quantity += unaccounted;
                            unaccounted = Decimal::ZERO;
                            self.lots.push_front(fifo_lot)
                        } else {
                            unaccounted += fifo_lot.quantity
                        }
                    }
                    None => {
                        // No more lots left, so now we want to push remaining qty onto lots
                        self.lots.push_back(Lot {
                            quantity: unaccounted,
                            ..new_lot
                        });
                        unaccounted = Decimal::ZERO;
                    }
                }
            }
        }
    }

    pub fn quantity(&self) -> Decimal {
        self.lots
            .iter()
            .fold(Decimal::ZERO, |acc, lot| acc + lot.quantity)
    }

    pub fn cost_basis(&self) -> Decimal {
        self.lots
            .iter()
            .fold(Decimal::ZERO, |acc, lot| acc + lot.quantity * lot.price)
    }

    pub fn average_price(&self) -> Option<Decimal> {
        let qty = self.quantity();
        if qty.is_zero() {
            None
        } else {
            Some((self.cost_basis() / qty).round_dp(8))
        }
    }

    pub fn market_value(&self, price: Decimal) -> Decimal {
        self.quantity() * price
    }

    pub fn unrealized_profit(&self, price: Decimal) -> Decimal {
        self.market_value(price) - self.cost_basis()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_do_fifo_lot_aggregation() {
        let mut price = Decimal::new(100, 0);
        let mut position = Position::new(
            "AAPL".to_string(),
            Lot {
                fill_time: Utc::now(),
                price,
                quantity: Decimal::new(2, 0),
            },
        );
        assert_eq!(position.quantity(), Decimal::new(2, 0));
        assert_eq!(position.cost_basis(), Decimal::new(200, 0));
        assert_eq!(position.average_price(), Some(price));
        assert_eq!(position.market_value(price), Decimal::new(200, 0));
        assert_eq!(position.unrealized_profit(price), Decimal::ZERO);

        price = Decimal::new(150, 0);
        position.add_lot(Lot {
            fill_time: Utc::now(),
            price,
            quantity: Decimal::new(3, 0),
        });
        assert_eq!(position.quantity(), Decimal::new(5, 0));
        assert_eq!(position.cost_basis(), Decimal::new(650, 0));
        assert_eq!(position.average_price(), Some(Decimal::new(130, 0)));
        assert_eq!(position.market_value(price), Decimal::new(750, 0));
        assert_eq!(position.unrealized_profit(price), Decimal::new(100, 0));

        price = Decimal::new(120, 0);
        position.add_lot(Lot {
            fill_time: Utc::now(),
            price,
            quantity: Decimal::new(-1, 0),
        });
        assert_eq!(position.quantity(), Decimal::new(4, 0));
        assert_eq!(position.cost_basis(), Decimal::new(550, 0));
        assert_eq!(position.average_price(), Some(Decimal::new(1375, 1)));
        assert_eq!(position.market_value(price), Decimal::new(480, 0));
        assert_eq!(position.unrealized_profit(price), Decimal::new(-70, 0));

        position.add_lot(Lot {
            fill_time: Utc::now(),
            price,
            quantity: Decimal::new(-3, 0),
        });
        assert_eq!(position.quantity(), Decimal::new(1, 0));
        assert_eq!(position.cost_basis(), Decimal::new(150, 0));
        assert_eq!(position.average_price(), Some(Decimal::new(150, 0)));
        assert_eq!(position.market_value(price), Decimal::new(120, 0));
        assert_eq!(position.unrealized_profit(price), Decimal::new(-30, 0));

        position.add_lot(Lot {
            fill_time: Utc::now(),
            price,
            quantity: Decimal::new(-3, 0),
        });
        assert_eq!(position.quantity(), Decimal::new(-2, 0));
        assert_eq!(position.cost_basis(), Decimal::new(-240, 0));
        assert_eq!(position.average_price(), Some(Decimal::new(120, 0)));
        assert_eq!(position.market_value(price), Decimal::new(-240, 0));
        assert_eq!(position.unrealized_profit(price), Decimal::ZERO);

        price = Decimal::new(80, 0);
        position.add_lot(Lot {
            fill_time: Utc::now(),
            price,
            quantity: Decimal::new(2, 0),
        });
        assert_eq!(position.quantity(), Decimal::ZERO);
        assert_eq!(position.cost_basis(), Decimal::ZERO);
        assert_eq!(position.average_price(), None);
        assert_eq!(position.market_value(price), Decimal::ZERO);
        assert_eq!(position.unrealized_profit(price), Decimal::ZERO);
    }
}
