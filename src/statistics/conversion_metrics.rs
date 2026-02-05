use crate::order::{Order, OrderStatus};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};
use std::fmt::{Display, Formatter};

pub struct ConversionMetrics {
    paid_count: usize,
    cancelled_count: usize,
    refunded_count: usize,
    paid_amount: f64,
    total_amount: f64,
}

impl ConversionMetrics {
    pub fn new() -> Self {
        Self {
            paid_count: 0,
            cancelled_count: 0,
            refunded_count: 0,
            paid_amount: 0.0,
            total_amount: 0.0,
        }
    }

    fn total_orders(&self) -> usize {
        self.paid_count + self.cancelled_count + self.refunded_count
    }

    fn paid_rate(&self) -> f64 {
        if self.total_orders() > 0 {
            (self.paid_count as f64 / self.total_orders() as f64) * 100.0
        } else {
            0.0
        }
    }

    fn cancellation_rate(&self) -> f64 {
        if self.total_orders() > 0 {
            (self.cancelled_count as f64 / self.total_orders() as f64) * 100.0
        } else {
            0.0
        }
    }

    fn refund_rate(&self) -> f64 {
        if self.total_orders() > 0 {
            (self.refunded_count as f64 / self.total_orders() as f64) * 100.0
        } else {
            0.0
        }
    }

    fn revenue_capture_rate(&self) -> f64 {
        if self.total_amount > 0.0 {
            (self.paid_amount / self.total_amount) * 100.0
        } else {
            0.0
        }
    }

    fn lost_revenue(&self) -> f64 {
        self.total_amount - self.paid_amount
    }
}

impl ConversionMetrics {
    pub fn accept(&mut self, order: &Order) {
        self.total_amount += order.amount;

        match order.status {
            OrderStatus::Paid => {
                self.paid_count += 1;
                self.paid_amount += order.amount;
            }
            OrderStatus::Cancelled => {
                self.cancelled_count += 1;
            }
            OrderStatus::Refunded => {
                self.refunded_count += 1;
            }
        }
    }
}

impl Display for ConversionMetrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "--- Conversion Metrics ---")?;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        table.set_header(vec!["Metric", "Value"]);
        table.add_row(vec![
            "Paid Rate".to_string(),
            format!("{:.2} %", self.paid_rate()),
        ]);
        table.add_row(vec![
            "Cancellation Rate".to_string(),
            format!("{:.2} %", self.cancellation_rate()),
        ]);
        table.add_row(vec![
            "Refund Rate".to_string(),
            format!("{:.2} %", self.refund_rate()),
        ]);
        table.add_row(vec![
            "Revenue Capture".to_string(),
            format!("{:.2} %", self.revenue_capture_rate()),
        ]);
        table.add_row(vec![
            "Actual Revenue".to_string(),
            format!("${:.2}", self.paid_amount),
        ]);
        table.add_row(vec![
            "Potential Revenue".to_string(),
            format!("${:.2}", self.total_amount),
        ]);
        table.add_row(vec![
            "Lost Revenue".to_string(),
            format!("${:.2}", self.lost_revenue()),
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
    fn new_metrics_is_empty() {
        let metrics = ConversionMetrics::new();

        assert_eq!(metrics.total_orders(), 0);
        assert!((metrics.paid_rate() - 0.0).abs() < f64::EPSILON);
        assert!((metrics.cancellation_rate() - 0.0).abs() < f64::EPSILON);
        assert!((metrics.refund_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_tracks_paid_orders() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 200.0, OrderStatus::Paid));

        assert_eq!(metrics.paid_count, 2);
        assert!((metrics.paid_amount - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn accept_tracks_cancelled_orders() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Cancelled));

        assert_eq!(metrics.cancelled_count, 1);
    }

    #[test]
    fn accept_tracks_refunded_orders() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Refunded));

        assert_eq!(metrics.refunded_count, 1);
    }

    #[test]
    fn paid_rate_calculates_correctly() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(3, "Bob", 100.0, OrderStatus::Cancelled));
        metrics.accept(&create_order(4, "Alice", 100.0, OrderStatus::Refunded));

        assert!((metrics.paid_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cancellation_rate_calculates_correctly() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 100.0, OrderStatus::Cancelled));
        metrics.accept(&create_order(3, "Bob", 100.0, OrderStatus::Cancelled));
        metrics.accept(&create_order(4, "Alice", 100.0, OrderStatus::Refunded));

        assert!((metrics.cancellation_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn refund_rate_calculates_correctly() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 100.0, OrderStatus::Refunded));

        assert!((metrics.refund_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn revenue_capture_rate_calculates_correctly() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 100.0, OrderStatus::Cancelled));

        // Paid: 100, Total: 200 -> 50%
        assert!((metrics.revenue_capture_rate() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lost_revenue_calculates_correctly() {
        let mut metrics = ConversionMetrics::new();

        metrics.accept(&create_order(1, "John", 100.0, OrderStatus::Paid));
        metrics.accept(&create_order(2, "Jane", 150.0, OrderStatus::Cancelled));
        metrics.accept(&create_order(3, "Bob", 50.0, OrderStatus::Refunded));

        // Total: 300, Paid: 100 -> Lost: 200
        assert!((metrics.lost_revenue() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn render_contains_header() {
        let metrics = ConversionMetrics::new();
        let output = metrics.to_string();

        assert!(output.contains("--- Conversion Metrics ---"));
    }
}
