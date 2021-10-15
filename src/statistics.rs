use crate::brokerage::{Event, OrderStatus};
use rust_decimal::Decimal;

#[derive(Default, Debug)]
pub struct OrderCounts {
    submitted: usize,
    cancelled: usize,
    filled: usize,
    rejected: usize,
}

#[derive(Debug)]
pub struct Statistics {
    order_counts: OrderCounts,
    commission_paid: Decimal,
    equity: Vec<Decimal>,
    event_log: Vec<Event>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            order_counts: OrderCounts::default(),
            commission_paid: Decimal::ZERO,
            equity: Vec::new(),
            event_log: Vec::new(),
        }
    }

    pub fn record_event(&mut self, event: Event) {
        self.event_log.push(event)
    }

    pub fn handle_order(&mut self, status: &OrderStatus) {
        match status {
            OrderStatus::Submitted => self.order_counts.submitted += 1,
            OrderStatus::Cancelled => self.order_counts.cancelled += 1,
            OrderStatus::Filled { .. } => self.order_counts.filled += 1,
            OrderStatus::Rejected => self.order_counts.rejected += 1,
            OrderStatus::PartiallyFilled => (),
        }
    }

    pub fn record_equity(&mut self, equity: Decimal) {
        self.equity.push(equity)
    }

    pub fn increase_commission(&mut self, amount: Decimal) {
        self.commission_paid += amount
    }
}
