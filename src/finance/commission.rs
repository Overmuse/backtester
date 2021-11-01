use crate::brokerage::position::Lot;
use rust_decimal::prelude::*;

pub trait Commission {
    fn calculate(&self, lot: &Lot) -> Decimal;
}

pub struct NoCommission;
impl Commission for NoCommission {
    fn calculate(&self, _: &Lot) -> Decimal {
        Decimal::ZERO
    }
}

pub struct PerShareCommission {
    amount: Decimal,
    min_lot_cost: Option<Decimal>,
}
impl PerShareCommission {
    pub fn new(amount: Decimal) -> Self {
        Self {
            amount,
            min_lot_cost: None,
        }
    }

    pub fn min_lot_cost(mut self, min_lot_cost: Decimal) -> Self {
        self.min_lot_cost = Some(min_lot_cost);
        self
    }
}
impl Commission for PerShareCommission {
    fn calculate(&self, lot: &Lot) -> Decimal {
        Decimal::max(
            self.amount * lot.quantity,
            self.min_lot_cost.unwrap_or_default(),
        )
    }
}

pub struct PerLotCommission {
    amount: Decimal,
}
impl PerLotCommission {
    pub fn new(amount: Decimal) -> Self {
        Self { amount }
    }
}
impl Commission for PerLotCommission {
    fn calculate(&self, _: &Lot) -> Decimal {
        self.amount
    }
}

pub struct PerDollarCommission {
    amount: Decimal,
    min_lot_cost: Option<Decimal>,
}
impl PerDollarCommission {
    pub fn new(amount: Decimal) -> Self {
        Self {
            amount,
            min_lot_cost: None,
        }
    }

    pub fn min_lot_cost(mut self, min_lot_cost: Decimal) -> Self {
        self.min_lot_cost = Some(min_lot_cost);
        self
    }
}
impl Commission for PerDollarCommission {
    fn calculate(&self, lot: &Lot) -> Decimal {
        Decimal::max(
            self.amount * lot.quantity * lot.price,
            self.min_lot_cost.unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;

    #[test]
    fn it_calculates_the_correct_commission_amount() {
        let no_commission = NoCommission;
        let per_share_commission = PerShareCommission::new(Decimal::new(1, 0));
        let per_lot_commission = PerLotCommission::new(Decimal::new(2, 0));
        let per_dollar_commission = PerDollarCommission::new(Decimal::new(3, 0));

        let lot = Lot {
            fill_time: Utc::now(),
            quantity: Decimal::new(4, 0),
            price: Decimal::new(5, 0),
        };

        assert_eq!(no_commission.calculate(&lot), Decimal::new(0, 0));
        assert_eq!(per_share_commission.calculate(&lot), Decimal::new(4, 0));
        assert_eq!(per_lot_commission.calculate(&lot), Decimal::new(2, 0));
        assert_eq!(per_dollar_commission.calculate(&lot), Decimal::new(60, 0));
    }

    #[test]
    fn it_respects_commission_minimums() {
        let per_share_commission =
            PerShareCommission::new(Decimal::new(1, 0)).min_lot_cost(Decimal::new(5, 0));
        let per_dollar_commission =
            PerDollarCommission::new(Decimal::new(3, 0)).min_lot_cost(Decimal::new(100, 0));

        let lot = Lot {
            fill_time: Utc::now(),
            quantity: Decimal::new(4, 0),
            price: Decimal::new(5, 0),
        };

        assert_eq!(per_share_commission.calculate(&lot), Decimal::new(5, 0));
        assert_eq!(per_dollar_commission.calculate(&lot), Decimal::new(100, 0));
    }
}
