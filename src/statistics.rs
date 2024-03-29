use crate::brokerage::actor::Event;
use crate::brokerage::order::OrderStatus;
use chrono::DateTime;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use std::fmt;

#[derive(Default, Debug)]
pub struct OrderCounts {
    submitted: usize,
    cancelled: usize,
    filled: usize,
    rejected: usize,
    expired: usize,
}

impl fmt::Display for OrderCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max = vec![
            self.submitted,
            self.cancelled,
            self.filled,
            self.rejected,
            self.expired,
        ]
        .into_iter()
        .max()
        .unwrap();
        let digits = f64::log10(max as f64).ceil() as usize;
        let full_digits = digits + 11;
        write!(
            f,
            r#"
{:=>full_digits$}
{: ^full_digits$}
{:=>full_digits$}
Submitted: {:>digits$}
Cancelled: {:>digits$}
Filled:    {:>digits$}
Rejected:  {:>digits$}
Expired:   {:>digits$}
        "#,
            "",
            "Orders",
            "",
            self.submitted,
            self.cancelled,
            self.filled,
            self.rejected,
            self.expired,
            full_digits = full_digits,
            digits = digits
        )
    }
}

#[derive(Debug)]
pub struct Statistics {
    order_counts: OrderCounts,
    commission_paid: Decimal,
    pub equity: Vec<(DateTime<Tz>, Decimal)>,
    pub event_log: Vec<Event>,
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
            OrderStatus::Expired => self.order_counts.expired += 1,
            OrderStatus::PartiallyFilled => (),
        }
    }

    pub fn record_equity(&mut self, datetime: DateTime<Tz>, equity: Decimal) {
        self.equity.push((datetime, equity));
    }

    pub fn increase_commission(&mut self, amount: Decimal) {
        self.commission_paid += amount
    }

    pub fn max_drawdown(&self) -> Decimal {
        #[derive(Default)]
        struct State {
            max_equity: Decimal,
            max_drawdown: Decimal,
        }

        self.equity
            .iter()
            .map(|(_, e)| e)
            .fold(State::default(), |mut state, equity| {
                if equity > &state.max_equity {
                    state.max_equity = *equity
                };
                let drawdown = equity / state.max_equity - Decimal::ONE;
                if drawdown < state.max_drawdown {
                    state.max_drawdown = drawdown;
                }
                state
            })
            .max_drawdown
    }
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.order_counts)?;
        write!(
            f,
            r#"
===============
  Commissions
===============
Paid: {:>9}
             "#,
            self.commission_paid.round_dp(2)
        )?;

        write!(
            f,
            r#"
===============
    Equity
===============
Starting: {:>.2}
Max:      {:>.2}
Min:      {:>.2}
Ending:   {:>.2}
            "#,
            self.equity.first().unwrap().1.round_dp(2),
            self.equity.iter().map(|x| x.1).max().unwrap().round_dp(2),
            self.equity.iter().map(|x| x.1).min().unwrap().round_dp(2),
            self.equity.last().unwrap().1.round_dp(2),
        )?;
        write!(
            f,
            r#"
===============
    Returns
===============
Total: {:>.2}%
            "#,
            (((self.equity.last().unwrap().1 / self.equity.first().unwrap().1) - Decimal::ONE)
                * Decimal::new(100, 0))
            .round_dp(2)
        )
        .unwrap();
        write!(
            f,
            r#"
===============
   Drawdowns
===============
Max: {}%
            "#,
            (self.max_drawdown() * Decimal::new(100, 0)).round_dp(2)
        )
    }
}
impl Default for Statistics {
    fn default() -> Self {
        Self::new()
    }
}
