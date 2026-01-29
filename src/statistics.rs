use crate::order::{Order, OrderStatus};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub struct Statistics {
    items_by_status: HashMap<OrderStatus, usize>,
    total_by_status: HashMap<OrderStatus, f64>,
    total_amount: f64,
    total_items: usize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            items_by_status: HashMap::new(),
            total_by_status: HashMap::new(),
            total_amount: 0.00,
            total_items: 0,
        }
    }

    pub fn accept(&mut self, order: Order) {
        self.increment_items_by_status(&order);
        self.add_total_by_status(&order);

        self.total_amount += order.amount;
        self.total_items += 1;
    }

    fn increment_items_by_status(&mut self, order: &Order) {
        *self.items_by_status.entry(order.status).or_insert(0) += 1;
    }

    fn add_total_by_status(&mut self, order: &Order) {
        *self.total_by_status.entry(order.status).or_insert(0.0) += order.amount;
    }

    pub fn count(&self, status: OrderStatus) -> usize {
        self.items_by_status.get(&status).copied().unwrap_or(0)
    }

    pub fn total(&self, status: OrderStatus) -> f64 {
        self.total_by_status.get(&status).copied().unwrap_or(0.0)
    }

    pub fn average(&self, status: OrderStatus) -> f64 {
        let count = self.count(status);
        if count > 0 {
            self.total(status) / count as f64
        } else {
            0.0
        }
    }
}

impl Display for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Orders statistics ===")?;
        writeln!(f)?;

        writeln!(
            f,
            "{:<12} {:>8} {:>12} {:>10}",
            "Status", "Count", "Total", "Avg"
        )?;
        writeln!(f, "-----------------------------------------")?;

        for status in [
            OrderStatus::Paid,
            OrderStatus::Cancelled,
            OrderStatus::Refunded,
        ] {
            writeln!(
                f,
                "{:<12} {:>8} {:>12.2} {:>10.2}",
                format!("{:?}", status),
                self.count(status),
                self.total(status),
                self.average(status),
            )?;
        }

        writeln!(f, "-----------------------------------------")?;

        let avg_all = if self.total_items > 0 {
            self.total_amount / self.total_items as f64
        } else {
            0.0
        };

        writeln!(
            f,
            "{:<12} {:>8} {:>12.2} {:>10.2}",
            "ALL", self.total_items, self.total_amount, avg_all
        )?;

        let paid_percent = if self.total_amount > 0.0 {
            self.total(OrderStatus::Paid) / self.total_amount * 100.0
        } else {
            0.0
        };

        writeln!(f, "\nPaid %: {:.2}%", paid_percent)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paid(amount: f64) -> Order {
        Order {
            id: 1,
            customer: "John".into(),
            amount,
            status: OrderStatus::Paid,
        }
    }

    fn cancelled(amount: f64) -> Order {
        Order {
            id: 1,
            customer: "John".into(),
            amount,
            status: OrderStatus::Cancelled,
        }
    }

    #[test]
    fn empty_statistics() {
        let stats = Statistics::new();

        assert_eq!(0, stats.count(OrderStatus::Paid));
        assert_eq!(0.0, stats.total(OrderStatus::Paid));
        assert_eq!(0.0, stats.average(OrderStatus::Paid));
    }

    #[test]
    fn single_order_updates_stats() {
        let mut stats = Statistics::new();

        stats.accept(paid(10.0));

        assert_eq!(1, stats.count(OrderStatus::Paid));
        assert_eq!(10.0, stats.total(OrderStatus::Paid));
        assert_eq!(10.0, stats.average(OrderStatus::Paid));
    }

    #[test]
    fn multiple_orders_same_status() {
        let mut stats = Statistics::new();

        stats.accept(paid(10.0));
        stats.accept(paid(30.0));

        assert_eq!(2, stats.count(OrderStatus::Paid));
        assert_eq!(40.0, stats.total(OrderStatus::Paid));
        assert_eq!(20.0, stats.average(OrderStatus::Paid));
    }

    #[test]
    fn multiple_statuses_are_independent() {
        let mut stats = Statistics::new();

        stats.accept(paid(10.0));
        stats.accept(cancelled(5.0));

        assert_eq!(1, stats.count(OrderStatus::Paid));
        assert_eq!(1, stats.count(OrderStatus::Cancelled));

        assert_eq!(10.0, stats.total(OrderStatus::Paid));
        assert_eq!(5.0, stats.total(OrderStatus::Cancelled));
    }

    #[test]
    fn average_returns_zero_when_no_items() {
        let stats = Statistics::new();

        assert_eq!(0.0, stats.average(OrderStatus::Paid));
    }

    #[test]
    fn display_contains_expected_values() {
        let mut stats = Statistics::new();

        stats.accept(paid(10.0));
        stats.accept(paid(30.0));
        stats.accept(cancelled(20.0));

        let output = format!("{}", stats);

        assert!(output.contains("Orders statistics"));
        assert!(output.contains("Paid"));
        assert!(output.contains("Cancelled"));

        // count
        assert!(output.contains("2"));
        assert!(output.contains("1"));

        // totals
        assert!(output.contains("40.00"));
        assert!(output.contains("20.00"));

        // average
        assert!(output.contains("20.00"));

        // ALL
        assert!(output.contains("60.00"));
    }

    #[test]
    fn display_does_not_panic_on_empty_stats() {
        let stats = Statistics::new();

        let output = format!("{}", stats);

        assert!(output.contains("0"));
    }
}
