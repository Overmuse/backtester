use super::order::Order;
use super::position::{Lot, Position};
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct Account {
    pub active_orders: Vec<Order>,
    pub inactive_orders: Vec<Order>,
    pub positions: HashMap<String, Position>,
    pub cash: Decimal,
    pub equity: Decimal,
    starting_cash: Decimal,
}

impl Account {
    pub fn new(cash: Decimal) -> Self {
        Self {
            active_orders: Vec::new(),
            inactive_orders: Vec::new(),
            positions: HashMap::new(),
            cash,
            equity: cash,
            starting_cash: cash,
        }
    }

    pub fn add_lot(&mut self, ticker: String, lot: Lot) {
        self.cash -= lot.price * lot.quantity;
        self.positions
            .entry(ticker.clone())
            .and_modify(|pos| pos.add_lot(lot.clone()))
            .or_insert_with(|| Position::new(ticker, lot));
    }

    pub fn reset(&mut self) {
        self.cash = self.starting_cash;
        self.equity = self.starting_cash;
        self.active_orders.clear();
        self.inactive_orders.clear();
        self.positions.clear()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_be_initialized() {
        let account = Account::new(Decimal::ONE_HUNDRED);
        assert!(account.active_orders.is_empty());
        assert!(account.inactive_orders.is_empty());
        assert!(account.positions.is_empty());
        assert_eq!(account.cash, Decimal::ONE_HUNDRED);
        assert_eq!(account.equity, Decimal::ONE_HUNDRED);
        assert_eq!(account.starting_cash, Decimal::ONE_HUNDRED);
    }

    #[test]
    fn it_can_be_reset() {
        let mut account = Account::new(Decimal::ONE_HUNDRED);
        account.cash = Decimal::new(200, 0);
        account.reset();
        assert_eq!(account.cash, Decimal::ONE_HUNDRED);
    }

    #[test]
    fn it_can_update_with_lots() {
        let mut account = Account::new(Decimal::ONE_HUNDRED);
        account.add_lot(
            "AAPL".into(),
            Lot {
                price: Decimal::new(2, 0),
                quantity: Decimal::new(3, 0),
            },
        );
        assert_eq!(account.cash, Decimal::new(94, 0));
        let pos = account.positions.get("AAPL");
        assert_eq!(pos.unwrap().quantity(), Decimal::new(3, 0));
    }
}
