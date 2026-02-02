use crate::order::{Order, OrderStatus};
use crate::statistics::Stat;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;

pub struct AmountsByStatus {
    items_by_status: HashMap<OrderStatus, usize>,
    total_by_status: HashMap<OrderStatus, f64>,
    total_amount: f64,
    total_items: usize,
}

impl AmountsByStatus {
    pub fn new() -> Self {
        Self {
            items_by_status: HashMap::new(),
            total_by_status: HashMap::new(),
            total_amount: 0.00,
            total_items: 0,
        }
    }

    fn increment_items_by_status(&mut self, order: &Order) {
        *self.items_by_status.entry(order.status).or_insert(0) += 1;
    }

    fn add_total_by_status(&mut self, order: &Order) {
        *self.total_by_status.entry(order.status).or_insert(0.0) += order.amount;
    }

    fn count(&self, status: OrderStatus) -> usize {
        self.items_by_status.get(&status).copied().unwrap_or(0)
    }

    fn total(&self, status: OrderStatus) -> f64 {
        self.total_by_status.get(&status).copied().unwrap_or(0.0)
    }

    fn average_by_status(&self, status: OrderStatus) -> f64 {
        let count = self.count(status);
        if count > 0 {
            self.total(status) / count as f64
        } else {
            0.0
        }
    }

    fn percentage_by_status(&self, status: OrderStatus) -> f64 {
        let count = self.count(status);
        if count > 0 {
            (count as f64 / self.total_items as f64) * 100.0
        } else {
            0.0
        }
    }

    fn average(&self) -> f64 {
        if self.total_items > 0 {
            self.total_amount / self.total_items as f64
        } else {
            0.0
        }
    }
}

impl Stat for AmountsByStatus {
    fn accept(&mut self, order: &Order) {
        self.increment_items_by_status(&order);
        self.add_total_by_status(&order);

        self.total_amount += order.amount;
        self.total_items += 1;
    }
}

impl Display for AmountsByStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Amounts by status ---")?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Status", "Count", "Total", "Average", "Percentage"]);

        for status in OrderStatus::iter() {
            table.add_row(vec![
                status.to_string(),
                self.count(status).to_string(),
                format!("{:.2}", self.total(status)),
                format!("{:.2}", self.average_by_status(status)),
                format!("{:.2} %", self.percentage_by_status(status)),
            ]);
        }

        table.add_row(vec![
            Cell::new("All")
                .bg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new(self.total_items)
                .bg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}", self.total_amount))
                .bg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new(format!("{:.2}", self.average()))
                .bg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("100.00 %")
                .bg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);

        writeln!(f, "{table}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_order(id: u32, customer: &str, amount: f64, status: OrderStatus) -> Order {
        Order {
            id,
            customer: customer.to_string(),
            amount,
            status,
        }
    }

    #[test]
    fn new_amounts_by_status_is_empty() {
        let stats = AmountsByStatus::new();

        assert_eq!(stats.total_items, 0);
        assert!((stats.total_amount - 0.0).abs() < f64::EPSILON);
        assert_eq!(stats.count(OrderStatus::Paid), 0);
        assert_eq!(stats.count(OrderStatus::Cancelled), 0);
        assert_eq!(stats.count(OrderStatus::Refunded), 0);
    }

    #[test]
    fn accept_increments_count_by_status() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        stats.accept(&create_order(3, "Bob", 30.0, OrderStatus::Cancelled));

        assert_eq!(stats.count(OrderStatus::Paid), 2);
        assert_eq!(stats.count(OrderStatus::Cancelled), 1);
        assert_eq!(stats.count(OrderStatus::Refunded), 0);
    }

    #[test]
    fn accept_tracks_total_by_status() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        stats.accept(&create_order(3, "Bob", 30.0, OrderStatus::Cancelled));

        assert!((stats.total(OrderStatus::Paid) - 30.0).abs() < f64::EPSILON);
        assert!((stats.total(OrderStatus::Cancelled) - 30.0).abs() < f64::EPSILON);
        assert!((stats.total(OrderStatus::Refunded) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_tracks_overall_totals() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 20.0, OrderStatus::Cancelled));
        stats.accept(&create_order(3, "Bob", 30.0, OrderStatus::Refunded));

        assert_eq!(stats.total_items, 3);
        assert!((stats.total_amount - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_by_status_calculates_correctly() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 30.0, OrderStatus::Paid));

        assert!((stats.average_by_status(OrderStatus::Paid) - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_by_status_returns_zero_for_empty() {
        let stats = AmountsByStatus::new();

        assert!((stats.average_by_status(OrderStatus::Paid) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentage_by_status_calculates_correctly() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 20.0, OrderStatus::Paid));
        stats.accept(&create_order(3, "Bob", 30.0, OrderStatus::Cancelled));
        stats.accept(&create_order(4, "Alice", 40.0, OrderStatus::Refunded));

        assert!((stats.percentage_by_status(OrderStatus::Paid) - 50.0).abs() < f64::EPSILON);
        assert!((stats.percentage_by_status(OrderStatus::Cancelled) - 25.0).abs() < f64::EPSILON);
        assert!((stats.percentage_by_status(OrderStatus::Refunded) - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentage_by_status_returns_zero_for_empty() {
        let stats = AmountsByStatus::new();

        assert!((stats.percentage_by_status(OrderStatus::Paid) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_calculates_overall_average() {
        let mut stats = AmountsByStatus::new();

        stats.accept(&create_order(1, "John", 10.0, OrderStatus::Paid));
        stats.accept(&create_order(2, "Jane", 20.0, OrderStatus::Cancelled));
        stats.accept(&create_order(3, "Bob", 30.0, OrderStatus::Refunded));

        assert!((stats.average() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_returns_zero_for_empty() {
        let stats = AmountsByStatus::new();

        assert!((stats.average() - 0.0).abs() < f64::EPSILON);
    }
}
